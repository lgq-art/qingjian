//! 局域网共享剪贴板调度协调器。

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

use qingjian_platform::ClipboardConfig;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::crypto::{Cipher, derive_key};
use crate::error::ClipboardError;
use crate::filter::{ClipboardHistory, LoopSuppressor, hash_payload};
use crate::model::ClipboardItem;
use crate::net::{DiscoveryService, TcpClient, TcpServer};
use crate::service::handle::ClipboardHandle;
use crate::watcher::{ClipboardMonitor, ClipboardWriter};

/// 局域网共享剪贴板核心服务。
pub struct ClipboardService;

impl ClipboardService {
    /// 启动剪贴板服务后台线程与异步任务，返回可供外部交互的 [`ClipboardHandle`]。
    pub fn spawn(config: &ClipboardConfig) -> Result<ClipboardHandle, ClipboardError> {
        let device_id = uuid::Uuid::new_v4().to_string();
        let device_name = config.effective_device_name();
        let key = derive_key(&config.passphrase);
        let cipher = Cipher::new(&key);

        let suppressor = Arc::new(LoopSuppressor::default());
        let history = Arc::new(ClipboardHistory::new(config.history_size));
        let writer = Arc::new(ClipboardWriter::new());

        let (manual_tx, manual_rx) = mpsc::unbounded_channel::<String>();
        let (local_tx, local_rx) = mpsc::unbounded_channel::<(String, u64)>();
        let (incoming_tx, incoming_rx) = mpsc::channel::<ClipboardItem>(64);

        // 启动本地系统剪贴板监听
        let monitor_suppressor = Arc::clone(&suppressor);
        let monitor_tx = local_tx.clone();
        let _monitor = ClipboardMonitor::start(
            monitor_suppressor,
            config.max_size_bytes,
            move |text, hash| {
                let _ = monitor_tx.send((text, hash));
            },
        );

        // 处理主动 push_text 的转发通道
        let manual_to_local_tx = local_tx.clone();
        tokio::spawn_or_thread(manual_rx, move |text| {
            let hash = hash_payload(text.as_bytes());
            let _ = manual_to_local_tx.send((text, hash));
        });

        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<Result<Arc<DiscoveryService>, String>>(1);

        let s_device_id = device_id.clone();
        let s_device_name = device_name.clone();
        let s_port = config.port;
        let s_cipher = cipher.clone();
        let s_max_size = config.max_size_bytes;
        let s_auto_sync = config.auto_sync;
        let s_static_peers: Vec<SocketAddr> = config
            .static_peers
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();

        let s_suppressor = Arc::clone(&suppressor);
        let s_history = Arc::clone(&history);

        // 在专属后台线程中运行 tokio 异步网络运行时
        thread::Builder::new()
            .name("qingjian-clipboard-service".to_string())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e.to_string()));
                        return;
                    }
                };

                rt.block_on(async move {
                    // 1. 启动 TCP 接收服务端
                    if let Err(e) = TcpServer::start(s_port, s_cipher.clone(), s_max_size, incoming_tx).await {
                        tracing::error!(%e, port = s_port, "TCP 剪贴板监听服务启动失败");
                        let _ = ready_tx.send(Err(e.to_string()));
                        return;
                    }

                    // 2. 启动 UDP 节点发现
                    let discovery = match DiscoveryService::start(
                        s_device_id.clone(),
                        s_device_name.clone(),
                        s_port,
                    ).await {
                        Ok(d) => Arc::new(d),
                        Err(e) => {
                            tracing::error!(%e, port = s_port, "UDP 局域网节点发现服务启动失败");
                            let _ = ready_tx.send(Err(e.to_string()));
                            return;
                        }
                    };

                    let _ = ready_tx.send(Ok(Arc::clone(&discovery)));
                    tracing::info!(
                        device = %s_device_name,
                        port = s_port,
                        "青简局域网共享剪贴板服务已就绪"
                    );

                    // 3. 事件主循环
                    event_loop(
                        s_device_id,
                        s_device_name,
                        s_cipher,
                        discovery,
                        s_static_peers,
                        s_suppressor,
                        s_history,
                        writer,
                        s_auto_sync,
                        local_rx,
                        incoming_rx,
                    ).await;
                });
            })
            .map_err(|e| ClipboardError::Io(e))?;

        let discovery = ready_rx
            .recv()
            .map_err(|_| ClipboardError::Backend("Clipboard runtime thread terminated prematurely".to_string()))?
            .map_err(|e| ClipboardError::Backend(e))?;

        Ok(ClipboardHandle::new(
            device_id,
            device_name,
            history,
            discovery,
            manual_tx,
        ))
    }
}

trait SpawnOrThread {
    fn spawn_or_thread<F: Fn(String) + Send + 'static>(rx: UnboundedReceiver<String>, handler: F);
}

impl SpawnOrThread for tokio::sync::mpsc::UnboundedReceiver<String> {
    fn spawn_or_thread<F: Fn(String) + Send + 'static>(mut rx: UnboundedReceiver<String>, handler: F) {
        thread::spawn(move || {
            while let Some(msg) = rx.blocking_recv() {
                handler(msg);
            }
        });
    }
}

async fn event_loop(
    my_id: String,
    my_name: String,
    cipher: Cipher,
    discovery: Arc<DiscoveryService>,
    static_peers: Vec<SocketAddr>,
    suppressor: Arc<LoopSuppressor>,
    history: Arc<ClipboardHistory>,
    writer: Arc<ClipboardWriter>,
    auto_sync: bool,
    mut local_rx: UnboundedReceiver<(String, u64)>,
    mut incoming_rx: mpsc::Receiver<ClipboardItem>,
) {
    loop {
        tokio::select! {
            // 本地剪贴板变动：打包并加密广播给局域网其他节点
            Some((text, hash)) = local_rx.recv() => {
                suppressor.suppress(hash);
                let item = ClipboardItem::from_text(&text, my_id.clone(), my_name.clone(), hash);
                history.push(item.clone());

                let mut targets = static_peers.clone();
                for peer in discovery.peers().await {
                    targets.push(peer.address);
                }

                if !targets.is_empty() {
                    tracing::debug!(
                        targets = targets.len(),
                        len = text.len(),
                        "向局域网节点推送剪贴板内容"
                    );
                    let _ = TcpClient::broadcast(&cipher, &item, &targets).await;
                }
            }

            // 接收到远端节点发来的条目：回声抑制检查 + 写入历史 + 可选写入系统剪贴板
            Some(item) = incoming_rx.recv() => {
                if item.sender_device_id == my_id {
                    continue;
                }
                if suppressor.is_suppressed(item.hash) {
                    continue;
                }

                suppressor.suppress(item.hash);
                tracing::info!(
                    from = %item.sender_device_name,
                    len = item.payload.len(),
                    "收到局域网剪贴板更新"
                );
                history.push(item.clone());

                if auto_sync {
                    if let Some(text) = item.as_text() {
                        if let Err(e) = writer.write_text(text, &suppressor) {
                            tracing::warn!(%e, "将局域网内容写入系统剪贴板失败");
                        } else {
                            tracing::debug!("已自动同步至系统剪贴板");
                        }
                    }
                }
            }
        }
    }
}

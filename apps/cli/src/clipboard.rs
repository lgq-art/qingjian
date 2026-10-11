//! 命令行剪贴板测试工具。

use std::sync::Arc;
use std::time::Duration;

use clap::{Args, Subcommand};
use qingjian_clipboard::filter::{LoopSuppressor, hash_payload};
use qingjian_clipboard::model::ClipboardItem;
use qingjian_clipboard::net::{DiscoveryService, TcpClient};
use qingjian_clipboard::watcher::ClipboardMonitor;
use qingjian_clipboard::{ClipboardService, derive_key};
use qingjian_platform::ClipboardConfig;

#[derive(Debug, Subcommand)]
pub enum ClipboardSubcommand {
    /// 监听本机系统剪贴板变动并打印
    Watch,

    /// 发现局域网内的其他青简节点
    Peers {
        /// 监听端口（默认 41258）
        #[arg(long, default_value_t = 41258)]
        port: u16,
    },

    /// 向局域网节点发送一段测试文本
    Send {
        /// 要发送的文本内容
        text: String,

        /// 目标节点地址（例如 127.0.0.1:41258，留空则自动广播给已发现的节点）
        #[arg(long)]
        target: Option<String>,

        /// 端口
        #[arg(long, default_value_t = 41258)]
        port: u16,

        /// 密钥
        #[arg(long, default_value = "")]
        passphrase: String,
    },

    /// 以独立守护模式运行局域网剪贴板服务（用于终端双节点互联验证）
    Serve {
        /// 设备名称
        #[arg(long)]
        name: Option<String>,

        /// 服务端口
        #[arg(long, default_value_t = 41258)]
        port: u16,

        /// 同步密钥
        #[arg(long, default_value = "")]
        passphrase: String,

        /// 是否自动写入系统剪贴板
        #[arg(long, default_value_t = true)]
        auto_sync: bool,
    },
}

pub fn handle(cmd: ClipboardSubcommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        ClipboardSubcommand::Watch => {
            println!("正在监听本机系统剪贴板变动，按 Ctrl+C 退出...");
            let suppressor = Arc::new(LoopSuppressor::default());
            let _monitor = ClipboardMonitor::start(suppressor, 5 * 1024 * 1024, |text, hash| {
                println!("[剪贴板变动] 哈希: {hash:#x}, 长度: {} 字符", text.len());
                println!("内容预览: {}", text.chars().take(80).collect::<String>());
            });
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        ClipboardSubcommand::Peers { port } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async move {
                println!("正在监听端口 {port} 上的 UDP 广播，等待 3 秒...");
                let my_id = uuid::Uuid::new_v4().to_string();
                let discovery = DiscoveryService::start(my_id, "CLI-Inspector".to_string(), port).await?;
                tokio::time::sleep(Duration::from_secs(3)).await;

                let peers = discovery.peers().await;
                if peers.is_empty() {
                    println!("未在局域网中发现其他在线节点。");
                } else {
                    println!("发现 {} 个在线节点：", peers.len());
                    for peer in peers {
                        println!(
                            "- 设备名: {} | IP: {} | 系统: {}",
                            peer.device_name, peer.address, peer.os
                        );
                    }
                }
                Ok(())
            })
        }
        ClipboardSubcommand::Send {
            text,
            target,
            port,
            passphrase,
        } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async move {
                let key = derive_key(&passphrase);
                let cipher = qingjian_clipboard::crypto::Cipher::new(&key);
                let hash = hash_payload(text.as_bytes());
                let item = ClipboardItem::from_text(
                    &text,
                    "CLI-Sender".to_string(),
                    "CLI-Sender".to_string(),
                    hash,
                );

                let targets = if let Some(target_str) = target {
                    vec![target_str.parse()?]
                } else {
                    println!("正在发现局域网节点...");
                    let discovery = DiscoveryService::start(
                        "CLI-Sender-Discovery".to_string(),
                        "CLI-Sender".to_string(),
                        port,
                    )
                    .await?;
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                    discovery.peers().await.into_iter().map(|p| p.address).collect()
                };

                if targets.is_empty() {
                    println!("未找到目标节点，发送取消。");
                } else {
                    println!("正在向 {} 个节点发送数据...", targets.len());
                    TcpClient::broadcast(&cipher, &item, &targets).await?;
                    println!("发送成功！");
                }
                Ok(())
            })
        }
        ClipboardSubcommand::Serve {
            name,
            port,
            passphrase,
            auto_sync,
        } => {
            let config = ClipboardConfig {
                enabled: true,
                device_name: name.unwrap_or_else(|| "CLI-Serve-Node".to_string()),
                passphrase,
                port,
                auto_sync,
                ..Default::default()
            };

            let _handle = ClipboardService::spawn(&config)?;
            println!(
                "青简局域网剪贴板服务正在运行（设备: {}, 端口: {}）...",
                config.effective_device_name(),
                config.port
            );
            println!("按 Ctrl+C 退出。");
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

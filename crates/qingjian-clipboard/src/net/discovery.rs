//! 局域网 UDP 广播节点发现与心跳注册。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::net::UdpSocket;
use tokio::sync::RwLock;

use crate::model::{BeaconPacket, PeerInfo};

/// 广播信标发送间隔（3 秒）。
pub const BEACON_INTERVAL: Duration = Duration::from_secs(3);

/// 节点心跳超时失效阈值（10 秒）。
pub const PEER_TIMEOUT: Duration = Duration::from_secs(10);

/// 局域网发现服务与在线节点注册表。
pub struct DiscoveryService {
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
}

impl DiscoveryService {
    /// 启动 UDP 发现服务，在后台自动收发心跳。
    pub async fn start(
        device_id: String,
        device_name: String,
        port: u16,
    ) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", port)).await?;
        socket.set_broadcast(true)?;

        let peers = Arc::new(RwLock::new(HashMap::new()));
        let registry_rx = Arc::clone(&peers);
        let registry_cleanup = Arc::clone(&peers);

        let socket_tx = Arc::new(socket);
        let socket_rx = Arc::clone(&socket_tx);

        let my_id = device_id.clone();
        let my_id_rx = device_id.clone();

        // 任务 1：定时广播自身信标
        tokio::spawn(async move {
            let broadcast_target: SocketAddr = ([255, 255, 255, 255], port).into();
            let mut interval = tokio::time::interval(BEACON_INTERVAL);
            loop {
                interval.tick().await;
                let beacon = BeaconPacket::new(my_id.clone(), device_name.clone(), port);
                if let Ok(bytes) = serde_json::to_vec(&beacon) {
                    let _ = socket_tx.send_to(&bytes, broadcast_target).await;
                }
            }
        });

        // 任务 2：接收来自局域网其他节点的广播
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                if let Ok((len, addr)) = socket_rx.recv_from(&mut buf).await {
                    if let Ok(beacon) = serde_json::from_slice::<BeaconPacket>(&buf[..len]) {
                        if beacon.is_valid() && beacon.device_id != my_id_rx {
                            let peer_addr = SocketAddr::new(addr.ip(), beacon.tcp_port);
                            let info = PeerInfo {
                                device_id: beacon.device_id.clone(),
                                device_name: beacon.device_name,
                                address: peer_addr,
                                os: beacon.os,
                                last_seen: Instant::now(),
                            };
                            let mut map = registry_rx.write().await;
                            map.insert(beacon.device_id, info);
                        }
                    }
                }
            }
        });

        // 任务 3：定期清理超时的离线节点
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                let mut map = registry_cleanup.write().await;
                let now = Instant::now();
                map.retain(|_, peer| now.duration_since(peer.last_seen) < PEER_TIMEOUT);
            }
        });

        Ok(Self { peers })
    }

    /// 获取当前局域网在线节点快照。
    pub async fn peers(&self) -> Vec<PeerInfo> {
        let map = self.peers.read().await;
        map.values().cloned().collect()
    }
}

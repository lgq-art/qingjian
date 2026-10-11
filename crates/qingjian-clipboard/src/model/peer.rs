//! 局域网在线节点信息。

use std::net::SocketAddr;
use std::time::Instant;

/// 局域网内已发现的青简输入法对端节点。
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// 节点设备唯一标识。
    pub device_id: String,

    /// 节点名称。
    pub device_name: String,

    /// 节点 TCP 传输服务监听地址（IP 与端口）。
    pub address: SocketAddr,

    /// 节点操作系统（"windows", "macos", "linux"）。
    pub os: String,

    /// 最近一次收到心跳广播的时间点。
    pub last_seen: Instant,
}

//! 局域网 UDP 发现信标数据包格式。

use serde::{Deserialize, Serialize};

/// 青简剪贴板发现协议魔数（QingJian ClipBoard）。
pub const BEACON_MAGIC: [u8; 4] = *b"QJCB";

/// 信标协议版本。
pub const BEACON_VERSION: u8 = 1;

/// 广播信标负载。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconPacket {
    /// 协议魔数。
    pub magic: [u8; 4],

    /// 协议版本。
    pub version: u8,

    /// 发送端设备唯一 ID。
    pub device_id: String,

    /// 发送端设备名称。
    pub device_name: String,

    /// 该节点开放的 TCP 服务端端口。
    pub tcp_port: u16,

    /// 操作系统标识（windows / macos / linux）。
    pub os: String,
}

impl BeaconPacket {
    /// 构建标准信标包。
    pub fn new(device_id: String, device_name: String, tcp_port: u16) -> Self {
        Self {
            magic: BEACON_MAGIC,
            version: BEACON_VERSION,
            device_id,
            device_name,
            tcp_port,
            os: std::env::consts::OS.to_string(),
        }
    }

    /// 检查协议魔数与版本是否合法。
    pub fn is_valid(&self) -> bool {
        self.magic == BEACON_MAGIC && self.version == BEACON_VERSION
    }
}

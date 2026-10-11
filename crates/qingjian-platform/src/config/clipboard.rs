//! 局域网共享剪贴板配置。

use serde::{Deserialize, Serialize};

/// 局域网共享剪贴板默认广播与通信端口。
pub const DEFAULT_CLIPBOARD_PORT: u16 = 41258;

/// 剪贴板历史默认保留条数。
pub const DEFAULT_HISTORY_SIZE: usize = 20;

/// 默认单次同步最大字节数（5 MB）。
pub const DEFAULT_MAX_SIZE_BYTES: usize = 5 * 1024 * 1024;

/// 局域网共享剪贴板配置分节 `[clipboard]`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipboardConfig {
    /// 是否开启局域网共享剪贴板。
    pub enabled: bool,

    /// 本机在局域网中显示的设备名称；留空则使用操作系统主机名。
    pub device_name: String,

    /// 局域网同步配对密钥；同一局域网内设备必须配置相同密钥才可解密与同步。
    pub passphrase: String,

    /// 局域网发现与通信端口。
    pub port: u16,

    /// 收到远端剪贴板内容后是否自动覆盖本机操作系统剪贴板；为 false 时仅存入输入法历史。
    pub auto_sync: bool,

    /// 剪贴板历史记录条数。
    pub history_size: usize,

    /// 单次同步最大字节数上限。
    pub max_size_bytes: usize,

    /// 静态远端节点地址列表（可选，例如 `["192.168.1.100:41258"]`），用于禁用广播的网络。
    pub static_peers: Vec<String>,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            device_name: String::new(),
            passphrase: String::new(),
            port: DEFAULT_CLIPBOARD_PORT,
            auto_sync: true,
            history_size: DEFAULT_HISTORY_SIZE,
            max_size_bytes: DEFAULT_MAX_SIZE_BYTES,
            static_peers: Vec::new(),
        }
    }
}

impl ClipboardConfig {
    /// 获取有效设备名称（配置为空时自动回退到计算机名）。
    pub fn effective_device_name(&self) -> String {
        if self.device_name.trim().is_empty() {
            std::env::var("COMPUTERNAME")
                .or_else(|_| std::env::var("HOSTNAME"))
                .unwrap_or_else(|_| "Qingjian-Node".to_string())
        } else {
            self.device_name.clone()
        }
    }
}

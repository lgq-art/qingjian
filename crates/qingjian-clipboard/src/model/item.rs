//! 共享剪贴板条目数据结构。

use serde::{Deserialize, Serialize};

use super::content_type::ContentType;

/// 局域网同步的单个剪贴板条目。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// 唯一标识 UUID。
    pub id: String,

    /// 产生时间戳（Unix 毫秒）。
    pub timestamp: u64,

    /// 发送端设备唯一 ID。
    pub sender_device_id: String,

    /// 发送端设备可读名称（例如 "MacBook-Pro", "Office-PC"）。
    pub sender_device_name: String,

    /// 数据类型。
    pub content_type: ContentType,

    /// 载荷字节数据（纯文本 UTF-8 或 PNG 图片）。
    pub payload: Vec<u8>,

    /// 载荷内容的快速哈希值（用于回声抑制与去重）。
    pub hash: u64,
}

impl ClipboardItem {
    /// 从纯文本创建条目。
    pub fn from_text(
        text: &str,
        sender_device_id: String,
        sender_device_name: String,
        hash: u64,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now,
            sender_device_id,
            sender_device_name,
            content_type: ContentType::PlainText,
            payload: text.as_bytes().to_vec(),
            hash,
        }
    }

    /// 若载荷为纯文本，则转换为字符串切片。
    pub fn as_text(&self) -> Option<&str> {
        match self.content_type {
            ContentType::PlainText => std::str::from_utf8(&self.payload).ok(),
            _ => None,
        }
    }
}

//! 剪贴板内容类型枚举。

use serde::{Deserialize, Serialize};

/// 剪贴板有效载荷内容类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// 纯文本（UTF-8 编码）。
    PlainText,

    /// PNG 图像数据（预留拓展类型）。
    ImagePng,
}

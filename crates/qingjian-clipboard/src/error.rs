//! 剪贴板组件错误类型。

use thiserror::Error;

/// 剪贴板操作与局域网同步错误。
#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Clipboard OS backend error: {0}")]
    Backend(String),

    #[error("Decryption failed: invalid passphrase or corrupted payload")]
    DecryptionFailed,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Payload exceeded limit of {0} bytes")]
    PayloadTooLarge(usize),

    #[error("Worker channel closed")]
    ChannelClosed,
}

//! 系统剪贴板写入器。

use std::sync::Mutex;

use arboard::Clipboard;

use crate::error::ClipboardError;
use crate::filter::{LoopSuppressor, hash_payload};

/// 线程安全的剪贴板写入器。
pub struct ClipboardWriter {
    clipboard: Mutex<Option<Clipboard>>,
}

impl Default for ClipboardWriter {
    fn default() -> Self {
        Self {
            clipboard: Mutex::new(Clipboard::new().ok()),
        }
    }
}

impl ClipboardWriter {
    /// 构造写入器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 向系统剪贴板写入纯文本，并向回声抑制器登记该内容的哈希，防止引发回发。
    pub fn write_text(&self, text: &str, suppressor: &LoopSuppressor) -> Result<(), ClipboardError> {
        let hash = hash_payload(text.as_bytes());
        suppressor.suppress(hash);

        let Ok(mut guard) = self.clipboard.lock() else {
            return Err(ClipboardError::Backend("Clipboard lock poisoned".to_string()));
        };

        // 如果之前的句柄失效，尝试重新初始化
        if guard.is_none() {
            *guard = Clipboard::new().ok();
        }

        let Some(ref mut cb) = *guard else {
            return Err(ClipboardError::Backend(
                "Unable to open system clipboard".to_string(),
            ));
        };

        cb.set_text(text.to_owned())
            .map_err(|e| ClipboardError::Backend(e.to_string()))
    }
}

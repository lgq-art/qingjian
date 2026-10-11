//! 系统剪贴板交互模块。

mod monitor;
mod writer;

pub use monitor::{ClipboardMonitor, POLL_INTERVAL};
pub use writer::ClipboardWriter;

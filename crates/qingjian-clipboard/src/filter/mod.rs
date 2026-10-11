//! 剪贴板过滤、回声抑制与历史记录管理。

mod history;
mod loop_suppressor;

pub use history::ClipboardHistory;
pub use loop_suppressor::{DEFAULT_SUPPRESS_TTL, LoopSuppressor, hash_payload};

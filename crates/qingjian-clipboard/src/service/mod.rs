//! 剪贴板服务层：生命周期编排与调度。

mod coordinator;
mod handle;

pub use coordinator::ClipboardService;
pub use handle::ClipboardHandle;

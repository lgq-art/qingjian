//! 系统剪贴板后台监听器。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use arboard::Clipboard;

use crate::filter::{LoopSuppressor, hash_payload};

/// 剪贴板轮询检查周期（350 毫秒）。
pub const POLL_INTERVAL: Duration = Duration::from_millis(350);

/// 后台剪贴板变动监听器。
pub struct ClipboardMonitor {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl ClipboardMonitor {
    /// 启动后台监听线程；当检测到非抑制的剪贴板变动时，调用 `on_change` 回调。
    pub fn start<F>(
        suppressor: Arc<LoopSuppressor>,
        max_size_bytes: usize,
        on_change: F,
    ) -> Self
    where
        F: Fn(String, u64) + Send + Sync + 'static,
    {
        let running = Arc::new(AtomicBool::new(true));
        let flag = Arc::clone(&running);

        let handle = thread::Builder::new()
            .name("qingjian-clipboard-monitor".to_string())
            .spawn(move || {
                let mut clipboard: Option<Clipboard> = Clipboard::new().ok();
                let mut last_hash: Option<u64> = None;

                while flag.load(Ordering::Relaxed) {
                    if clipboard.is_none() {
                        clipboard = Clipboard::new().ok();
                    }

                    if let Some(ref mut cb) = clipboard {
                        if let Ok(text) = cb.get_text() {
                            if !text.is_empty() && text.len() <= max_size_bytes {
                                let hash = hash_payload(text.as_bytes());
                                let changed = last_hash != Some(hash);

                                if changed {
                                    last_hash = Some(hash);
                                    if !suppressor.is_suppressed(hash) {
                                        on_change(text, hash);
                                    }
                                }
                            }
                        }
                    }

                    thread::sleep(POLL_INTERVAL);
                }
            })
            .ok();

        Self { running, handle }
    }

    /// 停止监听线程。
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for ClipboardMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

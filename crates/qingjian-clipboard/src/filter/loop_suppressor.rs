//! 回声抑制器：记录近期同步的哈希值，防止局域网接收后触发本地广播回环。

use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 默认抑制哈希保留 TTL（15 秒）。
pub const DEFAULT_SUPPRESS_TTL: Duration = Duration::from_secs(15);

/// 计算字节序列的快速哈希值。
pub fn hash_payload(data: &[u8]) -> u64 {
    let mut hasher = foldhash::fast::FixedState::default().build_hasher();
    hasher.write(data);
    hasher.finish()
}

/// 环路回声抑制器。
pub struct LoopSuppressor {
    suppressed: Mutex<HashMap<u64, Instant>>,
    ttl: Duration,
}

impl Default for LoopSuppressor {
    fn default() -> Self {
        Self::new(DEFAULT_SUPPRESS_TTL)
    }
}

impl LoopSuppressor {
    /// 构造指定 TTL 的抑制器。
    pub fn new(ttl: Duration) -> Self {
        Self {
            suppressed: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    /// 标记某个哈希在 TTL 时间内处于抑制状态（例如刚从网络收到并准备写入系统剪贴板）。
    pub fn suppress(&self, hash: u64) {
        if let Ok(mut map) = self.suppressed.lock() {
            map.insert(hash, Instant::now());
        }
    }

    /// 检查该哈希是否处于抑制状态；若已被抑制则返回 true，并在检查时清理过期条目。
    pub fn is_suppressed(&self, hash: u64) -> bool {
        let Ok(mut map) = self.suppressed.lock() else {
            return false;
        };
        let now = Instant::now();
        map.retain(|_, time| now.duration_since(*time) < self.ttl);
        map.contains_key(&hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suppress_and_check() {
        let suppressor = LoopSuppressor::new(Duration::from_millis(100));
        let hash = 123456789;

        assert!(!suppressor.is_suppressed(hash));
        suppressor.suppress(hash);
        assert!(suppressor.is_suppressed(hash));

        std::thread::sleep(Duration::from_millis(150));
        assert!(!suppressor.is_suppressed(hash));
    }
}

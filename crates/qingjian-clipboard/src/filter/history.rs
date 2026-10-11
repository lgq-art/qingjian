//! 剪贴板历史记录环形缓冲区。

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::model::ClipboardItem;

/// 线程安全的剪贴板历史记录容器。
pub struct ClipboardHistory {
    items: Mutex<VecDeque<ClipboardItem>>,
    capacity: usize,
}

impl ClipboardHistory {
    /// 构造指定容量上限的历史容器。
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity: capacity.max(1),
        }
    }

    /// 插入新条目；若最新一条内容哈希相同则替换更新时间戳，否则推入队头；超限时弹出最旧条目。
    pub fn push(&self, item: ClipboardItem) {
        let Ok(mut items) = self.items.lock() else {
            return;
        };

        // 如果第一条与待插入条目的内容哈希一致，则更新即可，避免连续重复
        if let Some(first) = items.front_mut() {
            if first.hash == item.hash {
                *first = item;
                return;
            }
        }

        if items.len() >= self.capacity {
            items.pop_back();
        }
        items.push_front(item);
    }

    /// 获取当前所有历史条目切片快照（按最新排在前）。
    pub fn list(&self) -> Vec<ClipboardItem> {
        let Ok(items) = self.items.lock() else {
            return Vec::new();
        };
        items.iter().cloned().collect()
    }

    /// 根据 ID 查找历史条目。
    pub fn get(&self, id: &str) -> Option<ClipboardItem> {
        let Ok(items) = self.items.lock() else {
            return None;
        };
        items.iter().find(|item| item.id == id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ContentType;

    fn sample_item(text: &str, hash: u64) -> ClipboardItem {
        ClipboardItem {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: 1000,
            sender_device_id: "test".to_string(),
            sender_device_name: "test".to_string(),
            content_type: ContentType::PlainText,
            payload: text.as_bytes().to_vec(),
            hash,
        }
    }

    #[test]
    fn history_capacity_limit() {
        let history = ClipboardHistory::new(2);
        history.push(sample_item("a", 1));
        history.push(sample_item("b", 2));
        history.push(sample_item("c", 3));

        let list = history.list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].hash, 3);
        assert_eq!(list[1].hash, 2);
    }
}

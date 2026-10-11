//! 剪贴板服务外部交互句柄。

use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::filter::ClipboardHistory;
use crate::model::{ClipboardItem, PeerInfo};
use crate::net::DiscoveryService;

/// 供外部查询与控制剪贴板服务的句柄。
#[derive(Clone)]
pub struct ClipboardHandle {
    device_id: String,
    device_name: String,
    history: Arc<ClipboardHistory>,
    discovery: Arc<DiscoveryService>,
    manual_tx: UnboundedSender<String>,
}

impl ClipboardHandle {
    pub(crate) fn new(
        device_id: String,
        device_name: String,
        history: Arc<ClipboardHistory>,
        discovery: Arc<DiscoveryService>,
        manual_tx: UnboundedSender<String>,
    ) -> Self {
        Self {
            device_id,
            device_name,
            history,
            discovery,
            manual_tx,
        }
    }

    /// 获取本机设备 ID。
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// 获取本机设备名。
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// 获取当前局域网在线节点列表。
    pub async fn peers(&self) -> Vec<PeerInfo> {
        self.discovery.peers().await
    }

    /// 获取最近同步的剪贴板历史记录。
    pub fn history(&self) -> Vec<ClipboardItem> {
        self.history.list()
    }

    /// 主动发送一段文本到局域网（例如命令行或特定快捷操作）。
    pub fn push_text(&self, text: &str) {
        let _ = self.manual_tx.send(text.to_owned());
    }
}

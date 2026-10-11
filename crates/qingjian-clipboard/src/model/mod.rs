//! 共享剪贴板核心数据结构与协议模型。

mod beacon;
mod content_type;
mod item;
mod peer;

pub use beacon::{BEACON_MAGIC, BEACON_VERSION, BeaconPacket};
pub use content_type::ContentType;
pub use item::ClipboardItem;
pub use peer::PeerInfo;

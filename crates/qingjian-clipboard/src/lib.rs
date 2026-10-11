//! 青简局域网共享剪贴板。
//!
//! 在局域网（Wi-Fi / LAN）内提供端到端加密的剪贴板发现与自动同步服务：
//!
//! - **零配置发现**：基于 UDP Beacon 心跳发现局域网内的其他青简节点；
//! - **端到端加密**：使用 ChaCha20-Poly1305 AEAD 算法，由用户 passphrase 派生密钥进行全量加密；
//! - **防回声风暴**：采用带 TTL 的哈希回声抑制器，杜绝两端相互回写引发的网络风暴；
//! - **系统剪贴板集成**：基于 `arboard` 跨平台适配 Windows、macOS 与 Linux；
//! - **非阻塞架构**：全异步独立线程运行，不干扰输入法按键与组句。

pub mod crypto;
pub mod error;
pub mod filter;
pub mod model;
pub mod net;
pub mod service;
pub mod watcher;

pub use error::ClipboardError;
pub use filter::{ClipboardHistory, LoopSuppressor, hash_payload};
pub use model::{BeaconPacket, ClipboardItem, ContentType, PeerInfo};
pub use service::{ClipboardHandle, ClipboardService};

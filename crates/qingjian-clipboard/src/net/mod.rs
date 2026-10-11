//! 局域网网络传输与发现模块。

mod client;
mod codec;
mod discovery;
mod server;

pub use client::TcpClient;
pub use codec::{MAX_FRAME_LEN, read_frame, write_frame};
pub use discovery::{BEACON_INTERVAL, DiscoveryService, PEER_TIMEOUT};
pub use server::TcpServer;

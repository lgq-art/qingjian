//! TCP 传输客户端：将本地剪贴板条目加密后并发推送给目标节点。

use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::crypto::Cipher;
use crate::error::ClipboardError;
use crate::model::ClipboardItem;
use crate::net::codec::write_frame;

/// 连接与写入超时时间（2 秒）。
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// 剪贴板 TCP 推送客户端。
pub struct TcpClient;

impl TcpClient {
    /// 加密条目并向指定的节点地址列表发送。
    pub async fn broadcast(
        cipher: &Cipher,
        item: &ClipboardItem,
        targets: &[SocketAddr],
    ) -> Result<(), ClipboardError> {
        let serialized = serde_json::to_vec(item)?;
        let encrypted = cipher.encrypt(&serialized)?;

        for &addr in targets {
            let data = encrypted.clone();
            tokio::spawn(async move {
                if let Ok(Ok(mut stream)) = timeout(CONNECT_TIMEOUT, TcpStream::connect(addr)).await {
                    let _ = timeout(CONNECT_TIMEOUT, write_frame(&mut stream, &data)).await;
                }
            });
        }

        Ok(())
    }
}

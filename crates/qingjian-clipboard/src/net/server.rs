//! TCP 传输服务端：接收局域网远端节点推送的加密剪贴板条目。

use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;

use crate::crypto::Cipher;
use crate::error::ClipboardError;
use crate::model::ClipboardItem;
use crate::net::codec::read_frame;

/// TCP 剪贴板服务端。
pub struct TcpServer;

impl TcpServer {
    /// 启动 TCP 服务端，接收到的合法条目通过通道发送。
    pub async fn start(
        port: u16,
        cipher: Cipher,
        max_size_bytes: usize,
        incoming_tx: Sender<ClipboardItem>,
    ) -> Result<(), ClipboardError> {
        let listener = TcpListener::bind(("0.0.0.0", port)).await?;

        tokio::spawn(async move {
            while let Ok((mut socket, peer_addr)) = listener.accept().await {
                let cipher = cipher.clone();
                let tx = incoming_tx.clone();

                tokio::spawn(async move {
                    if let Ok(encrypted_packet) = read_frame(&mut socket, max_size_bytes).await {
                        if let Ok(decrypted_bytes) = cipher.decrypt(&encrypted_packet) {
                            if let Ok(item) = serde_json::from_slice::<ClipboardItem>(&decrypted_bytes) {
                                tracing::debug!(
                                    from = %peer_addr,
                                    sender = %item.sender_device_name,
                                    "收到局域网剪贴板同步条目"
                                );
                                let _ = tx.send(item).await;
                            }
                        } else {
                            tracing::warn!(from = %peer_addr, "收到无法解密的数据包（密钥不匹配或被破坏）");
                        }
                    }
                });
            }
        });

        Ok(())
    }
}

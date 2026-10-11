//! 局域网共享剪贴板集成测试。

use qingjian_clipboard::crypto::{Cipher, derive_key};
use qingjian_clipboard::filter::{LoopSuppressor, hash_payload};
use qingjian_clipboard::model::{BeaconPacket, ClipboardItem, ContentType};
use qingjian_clipboard::net::{TcpClient, TcpServer};
use tokio::sync::mpsc;

#[test]
fn beacon_packet_validation() {
    let packet = BeaconPacket::new("dev-1".to_string(), "PC-1".to_string(), 41258);
    assert!(packet.is_valid());
    assert_eq!(packet.magic, *b"QJCB");
    assert_eq!(packet.version, 1);
}

#[tokio::test]
async fn loop_suppressor_prevents_echo() {
    let suppressor = LoopSuppressor::default();
    let text = "Hello from device A";
    let hash = hash_payload(text.as_bytes());

    assert!(!suppressor.is_suppressed(hash));
    suppressor.suppress(hash);
    assert!(suppressor.is_suppressed(hash));
}

#[tokio::test]
async fn direct_tcp_transfer_and_decryption() {
    let key = derive_key("test-secret-cluster");
    let cipher = Cipher::new(&key);

    let (incoming_tx, mut incoming_rx) = mpsc::channel::<ClipboardItem>(16);

    // 绑定本地随机空闲端口
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    TcpServer::start(port, cipher.clone(), 1024 * 1024, incoming_tx)
        .await
        .unwrap();

    let hash = hash_payload(b"Sync message over LAN");
    let item = ClipboardItem {
        id: "msg-123".to_string(),
        timestamp: 12345678,
        sender_device_id: "dev-A".to_string(),
        sender_device_name: "MacBook-A".to_string(),
        content_type: ContentType::PlainText,
        payload: b"Sync message over LAN".to_vec(),
        hash,
    };

    let target: std::net::SocketAddr = ([127, 0, 0, 1], port).into();
    TcpClient::broadcast(&cipher, &item, &[target]).await.unwrap();

    let received = tokio::time::timeout(std::time::Duration::from_secs(3), incoming_rx.recv())
        .await
        .expect("接收超时")
        .expect("通道已关闭");

    assert_eq!(received.id, "msg-123");
    assert_eq!(received.sender_device_name, "MacBook-A");
    assert_eq!(received.as_text(), Some("Sync message over LAN"));
    assert_eq!(received.hash, hash);
}

//! 基于 ChaCha20-Poly1305 的对称认证加密与解密。

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::RngCore;

use crate::error::ClipboardError;

/// Nonce 随机数长度（12 字节，96 位）。
pub const NONCE_LEN: usize = 12;

/// 密码包装器。
#[derive(Clone)]
pub struct Cipher {
    inner: ChaCha20Poly1305,
}

impl Cipher {
    /// 从 32 字节密钥构造加密器。
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::from_slice(key);
        Self {
            inner: ChaCha20Poly1305::new(key),
        }
    }

    /// 加密明文，返回 `[12 字节 Nonce] + [密文与 Poly1305 认证标签]`。
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, ClipboardError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut ciphertext = self
            .inner
            .encrypt(nonce, plaintext)
            .map_err(|_| ClipboardError::DecryptionFailed)?;

        let mut packet = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        packet.extend_from_slice(&nonce_bytes);
        packet.append(&mut ciphertext);
        Ok(packet)
    }

    /// 解密数据包；若长度不足、Nonce 损坏或认证标签不匹配，返回 `ClipboardError::DecryptionFailed`。
    pub fn decrypt(&self, packet: &[u8]) -> Result<Vec<u8>, ClipboardError> {
        if packet.len() < NONCE_LEN {
            return Err(ClipboardError::DecryptionFailed);
        }
        let (nonce_bytes, ciphertext) = packet.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.inner
            .decrypt(nonce, ciphertext)
            .map_err(|_| ClipboardError::DecryptionFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let cipher = Cipher::new(&key);
        let message = b"Hello, Qingjian LAN Clipboard!";

        let encrypted = cipher.encrypt(message).unwrap();
        assert_ne!(&encrypted[..], message);

        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(&decrypted[..], message);
    }

    #[test]
    fn wrong_key_fails_decryption() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let cipher1 = Cipher::new(&key1);
        let cipher2 = Cipher::new(&key2);

        let encrypted = cipher1.encrypt(b"Secret text").unwrap();
        let result = cipher2.decrypt(&encrypted);
        assert!(result.is_err());
    }
}

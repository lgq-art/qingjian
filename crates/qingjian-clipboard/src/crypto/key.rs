//! 密钥派生：从用户配置的 passphrase 计算 256 位对称密钥。

use sha2::{Digest, Sha256};

/// 派生密钥长度（32 字节，256 位）。
pub const KEY_LEN: usize = 32;

/// 当用户未指定 passphrase 时使用的默认集群混淆盐。
const DEFAULT_SALT: &[u8] = b"qingjian-lan-clipboard-salt-v1";

/// 从用户密码计算 32 字节对称加密密钥。
pub fn derive_key(passphrase: &str) -> [u8; KEY_LEN] {
    let mut hasher = Sha256::new();
    hasher.update(DEFAULT_SALT);
    hasher.update(passphrase.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&result);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_key_is_deterministic() {
        let key1 = derive_key("my-secret-123");
        let key2 = derive_key("my-secret-123");
        assert_eq!(key1, key2);

        let key3 = derive_key("other-secret");
        assert_ne!(key1, key3);
    }
}

//! 加解密与密钥安全模块。

mod cipher;
mod key;

pub use cipher::{Cipher, NONCE_LEN};
pub use key::{KEY_LEN, derive_key};

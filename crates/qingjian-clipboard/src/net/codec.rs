//! 网络分帧编解码：4 字节大端长度前缀 + 载荷数据。

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::ClipboardError;

/// 最大单帧长度限制（默认为 10 MB 安全边界）。
pub const MAX_FRAME_LEN: usize = 10 * 1024 * 1024;

/// 从异步流中读取一个完整帧。
pub async fn read_frame<R: AsyncRead + Unpin>(
    reader: &mut R,
    max_size: usize,
) -> Result<Vec<u8>, ClipboardError> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > max_size || len > MAX_FRAME_LEN {
        return Err(ClipboardError::PayloadTooLarge(len));
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;
    Ok(payload)
}

/// 向异步流中写入长度前缀与载荷。
pub async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8],
) -> Result<(), ClipboardError> {
    if payload.len() > MAX_FRAME_LEN {
        return Err(ClipboardError::PayloadTooLarge(payload.len()));
    }

    let len = (payload.len() as u32).to_be_bytes();
    writer.write_all(&len).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn frame_roundtrip() {
        let (mut client, mut server) = duplex(64);
        let sample = b"Test frame payload data";

        tokio::spawn(async move {
            write_frame(&mut client, sample).await.unwrap();
        });

        let received = read_frame(&mut server, 1024).await.unwrap();
        assert_eq!(&received[..], sample);
    }
}

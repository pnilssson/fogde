use std::io;

use tokio::io::{AsyncRead, AsyncReadExt};

pub const DATA: u8 = 0;
pub const CREDIT: u8 = 1;
pub const CLOSE: u8 = 2;
pub const RESIZE: u8 = 3;
pub const FOCUS: u8 = 4;
pub const FACTS: u8 = 5;
pub const AUTOMATIC: u8 = 6;

const HEADER: usize = 9;
const LIMIT: usize = 16 << 20;

#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub channel: u32,
    pub kind: u8,
    pub payload: Vec<u8>,
}

pub fn encode(channel: u32, kind: u8, payload: &[u8]) -> Vec<u8> {
    let length = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    let mut frame = Vec::with_capacity(HEADER + payload.len());
    frame.extend_from_slice(&channel.to_le_bytes());
    frame.push(kind);
    frame.extend_from_slice(&length.to_le_bytes());
    frame.extend_from_slice(payload);

    frame
}

pub fn size(cols: u16, rows: u16) -> [u8; 4] {
    let [a, b] = cols.to_le_bytes();
    let [c, d] = rows.to_le_bytes();

    [a, b, c, d]
}

pub async fn read(reader: &mut (impl AsyncRead + Unpin)) -> io::Result<Option<Frame>> {
    let mut header = [0; HEADER];
    match reader.read_exact(&mut header).await {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }

    let channel = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
    let kind = header[4];
    let length = u32::from_le_bytes([header[5], header[6], header[7], header[8]]) as usize;
    if length > LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Herdr sent a frame that is too large",
        ));
    }

    let mut payload = vec![0; length];
    reader.read_exact(&mut payload).await?;

    Ok(Some(Frame {
        channel,
        kind,
        payload,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reads_the_frames_it_writes() {
        let mut bytes = encode(1, DATA, b"hi");
        bytes.extend(encode(7, CLOSE, b""));
        let mut reader = bytes.as_slice();

        assert_eq!(
            read(&mut reader).await.unwrap(),
            Some(Frame {
                channel: 1,
                kind: DATA,
                payload: b"hi".to_vec(),
            })
        );
        assert_eq!(
            read(&mut reader).await.unwrap(),
            Some(Frame {
                channel: 7,
                kind: CLOSE,
                payload: Vec::new(),
            })
        );
        assert_eq!(read(&mut reader).await.unwrap(), None);
    }

    #[test]
    fn writes_the_header_as_herdr_reads_it() {
        assert_eq!(
            encode(2, RESIZE, &size(100, 12)),
            [2, 0, 0, 0, 3, 4, 0, 0, 0, 100, 0, 12, 0]
        );
    }

    #[tokio::test]
    async fn refuses_an_oversized_frame() {
        let mut header = encode(1, DATA, b"");
        header[5..9].copy_from_slice(&u32::MAX.to_le_bytes());

        assert!(read(&mut header.as_slice()).await.is_err());
    }
}

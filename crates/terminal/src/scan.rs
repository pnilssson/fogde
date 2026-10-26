const ESC: u8 = 0x1b;
const BEL: u8 = 0x07;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Piece<'a> {
    Text(&'a [u8]),
    Control(&'a [u8]),
}

pub(crate) fn next(bytes: &[u8]) -> Piece<'_> {
    let Some(&first) = bytes.first() else {
        return Piece::Text(bytes);
    };

    if printable(first) {
        let end = bytes
            .iter()
            .position(|&byte| !printable(byte))
            .unwrap_or(bytes.len());
        return Piece::Text(&bytes[..end]);
    }

    if first != ESC {
        return Piece::Control(&bytes[..1]);
    }

    Piece::Control(&bytes[..escape(bytes)])
}

fn printable(byte: u8) -> bool {
    byte >= 0x20 && byte != 0x7f
}

fn escape(bytes: &[u8]) -> usize {
    match bytes.get(1) {
        None => bytes.len(),
        Some(b'[') => bytes
            .iter()
            .skip(2)
            .position(|byte| (0x40..=0x7e).contains(byte))
            .map_or(bytes.len(), |end| end + 3),
        Some(b']' | b'P' | b'X' | b'^' | b'_') => string(bytes),
        Some(_) => bytes
            .iter()
            .skip(1)
            .position(|byte| (0x30..=0x7e).contains(byte))
            .map_or(bytes.len(), |end| end + 2),
    }
}

fn string(bytes: &[u8]) -> usize {
    let mut index = 2;
    while index < bytes.len() {
        match bytes[index] {
            BEL => return index + 1,
            ESC if bytes.get(index + 1) == Some(&b'\\') => return index + 2,
            _ => index += 1,
        }
    }

    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pieces(mut bytes: &[u8]) -> Vec<Piece<'_>> {
        let mut pieces = Vec::new();
        while !bytes.is_empty() {
            let piece = next(bytes);
            let (Piece::Text(taken) | Piece::Control(taken)) = piece;
            bytes = &bytes[taken.len()..];
            pieces.push(piece);
        }
        pieces
    }

    #[test]
    fn splits_herdr_output_into_text_and_sequences() {
        let bytes = "\x1b[1;1H\x1b[0;1;38;2;1;2;3;49m界a\x1b]8;;https://example.com\x1b\\b\x1b[?25l\r\n\x1bc".as_bytes();

        assert_eq!(
            pieces(bytes),
            [
                Piece::Control(b"\x1b[1;1H"),
                Piece::Control(b"\x1b[0;1;38;2;1;2;3;49m"),
                Piece::Text("界a".as_bytes()),
                Piece::Control(b"\x1b]8;;https://example.com\x1b\\"),
                Piece::Text(b"b"),
                Piece::Control(b"\x1b[?25l"),
                Piece::Control(b"\r"),
                Piece::Control(b"\n"),
                Piece::Control(b"\x1bc"),
            ]
        );
    }

    #[test]
    fn keeps_an_unfinished_sequence_whole() {
        assert_eq!(pieces(b"\x1b[38;5"), [Piece::Control(b"\x1b[38;5")]);
        assert_eq!(
            pieces(b"\x1b]8;;x\x07y"),
            [Piece::Control(b"\x1b]8;;x\x07"), Piece::Text(b"y")]
        );
    }
}

use std::fmt::Write as _;
use std::io;

pub fn generate() -> io::Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(io::Error::other)?;

    Ok(bytes.iter().fold(String::new(), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    }))
}

pub fn matches(given: &str, expected: &str) -> bool {
    if given.len() != expected.len() {
        return false;
    }

    given
        .bytes()
        .zip(expected.bytes())
        .fold(0, |difference, (a, b)| difference | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_distinct_tokens() {
        let first = generate().unwrap();
        let second = generate().unwrap();

        assert_eq!(first.len(), 64);
        assert_ne!(first, second);
    }

    #[test]
    fn matches_only_the_same_token() {
        assert!(matches("abc123", "abc123"));
        assert!(!matches("abc124", "abc123"));
        assert!(!matches("abc12", "abc123"));
        assert!(!matches("", "abc123"));
    }
}

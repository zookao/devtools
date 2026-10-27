//! Base64 编解码（标准字母表，UTF-8 文本）。

use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};
use base64::Engine as _;

pub fn encode(input: &str) -> String {
    STANDARD.encode(input.as_bytes())
}

/// 解码为 UTF-8 文本。忽略空白字符，兼容无 padding 的输入。
pub fn decode(input: &str) -> Result<String, String> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = STANDARD
        .decode(&cleaned)
        .or_else(|_| STANDARD_NO_PAD.decode(&cleaned))
        .map_err(|e| format!("Invalid Base64 input: {e}"))?;
    String::from_utf8(bytes).map_err(|_| "Decoded bytes are not valid UTF-8 text.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_ascii() {
        assert_eq!(encode("Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn encode_utf8() {
        assert_eq!(encode("中文"), "5Lit5paH");
    }

    #[test]
    fn round_trip() {
        let s = "DevTools — JSON 工具";
        assert_eq!(decode(&encode(s)).unwrap(), s);
    }

    #[test]
    fn decode_ignores_whitespace() {
        assert_eq!(decode("SGVsbG8s\nIFdvcmxk IQ==").unwrap(), "Hello, World!");
    }

    #[test]
    fn decode_without_padding() {
        assert_eq!(decode("5Lit5paH").unwrap(), "中文");
    }

    #[test]
    fn decode_invalid() {
        assert!(decode("!!!").is_err());
    }
}

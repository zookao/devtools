//! 哈希计算：对文本按 UTF-8 字节计算摘要。

use md5::{Digest, Md5};

/// 计算 MD5，返回 32 位小写十六进制。
pub fn md5_hex(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(32);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md5_empty() {
        assert_eq!(md5_hex(""), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn md5_ascii() {
        assert_eq!(md5_hex("abc"), "900150983cd24fb0d6963f7d28e17f72");
    }

    #[test]
    fn md5_utf8() {
        // "中文" 的 MD5
        assert_eq!(md5_hex("中文"), "a7bac2239fcdcb3a067903d8077c4a07");
    }
}

//! UUID v4 生成（RFC 4122：随机版本，128 位随机源）。

use rand::Rng;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UuidOptions {
    /// 生成数量
    pub count: usize,
    /// 十六进制字母大写
    pub uppercase: bool,
    /// 保留 8-4-4-4-12 的分段连字符
    pub hyphens: bool,
    /// 两端加花括号
    pub braces: bool,
}

impl Default for UuidOptions {
    fn default() -> Self {
        Self {
            count: 5,
            uppercase: false,
            hyphens: true,
            braces: false,
        }
    }
}

/// 生成一批 UUID v4
pub fn generate(o: &UuidOptions) -> Vec<String> {
    let count = o.count.clamp(1, 100);
    (0..count).map(|_| generate_one(o)).collect()
}

fn generate_one(o: &UuidOptions) -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill(&mut bytes);

    // version = 4
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    // variant = RFC 4122（10xx）
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let mut out = String::with_capacity(38);
    if o.braces {
        out.push('{');
    }
    for (i, b) in bytes.iter().enumerate() {
        if o.hyphens && matches!(i, 4 | 6 | 8 | 10) {
            out.push('-');
        }
        if o.uppercase {
            out.push_str(&format!("{b:02X}"));
        } else {
            out.push_str(&format!("{b:02x}"));
        }
    }
    if o.braces {
        out.push('}');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> UuidOptions {
        UuidOptions::default()
    }

    #[test]
    fn default_format_is_lowercase_with_hyphens() {
        let out = generate(&UuidOptions { count: 3, ..opts() });
        assert_eq!(out.len(), 3);
        for u in &out {
            assert_eq!(u.len(), 36, "{u}");
            assert!(u.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
            assert_eq!(u.as_bytes()[14] as char, '4', "version nibble: {u}");
            assert!(
                matches!(u.as_bytes()[19] as char, '8' | '9' | 'a' | 'b'),
                "variant nibble: {u}"
            );
        }
    }

    #[test]
    fn uppercase_without_hyphens() {
        let out = generate(&UuidOptions {
            count: 1,
            uppercase: true,
            hyphens: false,
            braces: false,
        });
        assert_eq!(out[0].len(), 32);
        assert!(out[0].chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[test]
    fn braces_are_added() {
        let out = generate(&UuidOptions {
            count: 1,
            braces: true,
            ..opts()
        });
        assert!(out[0].starts_with('{') && out[0].ends_with('}'));
        assert_eq!(out[0].len(), 38);
    }

    #[test]
    fn count_is_clamped() {
        assert_eq!(generate(&UuidOptions { count: 0, ..opts() }).len(), 1);
        assert_eq!(generate(&UuidOptions { count: 1000, ..opts() }).len(), 100);
    }

    #[test]
    fn values_are_unique() {
        let out = generate(&UuidOptions { count: 100, ..opts() });
        let unique: std::collections::HashSet<_> = out.iter().collect();
        assert_eq!(unique.len(), out.len());
    }
}

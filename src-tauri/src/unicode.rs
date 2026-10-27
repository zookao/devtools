//! Unicode 转义编解码：文本 ↔ `\uXXXX` 序列。

/// 文本 → `\uXXXX`。非 BMP 字符输出代理对（`\uD83D\uDE00` 形式）。
pub fn encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 6);
    for ch in input.chars() {
        let cp = ch as u32;
        if cp <= 0xFFFF {
            out.push_str(&format!("\\u{cp:04x}"));
        } else {
            let v = cp - 0x1_0000;
            let hi = 0xD800 + (v >> 10);
            let lo = 0xDC00 + (v & 0x3FF);
            out.push_str(&format!("\\u{hi:04x}\\u{lo:04x}"));
        }
    }
    out
}

/// `\uXXXX` → 文本。非转义内容原样保留，孤立代理项替换为 U+FFFD。
pub fn decode(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == 'u' && i + 6 <= chars.len() {
            let hex: String = chars[i + 2..i + 6].iter().collect();
            if let Ok(cp) = u32::from_str_radix(&hex, 16) {
                if (0xD800..=0xDBFF).contains(&cp) {
                    // 高代理：尝试与紧跟的低代理组合
                    if i + 12 <= chars.len()
                        && chars[i + 6] == '\\'
                        && chars[i + 7] == 'u'
                        && let Ok(lo) = u32::from_str_radix(
                            &chars[i + 8..i + 12].iter().collect::<String>(),
                            16,
                        )
                        && (0xDC00..=0xDFFF).contains(&lo)
                        && let Some(c) =
                            char::from_u32(0x1_0000 + ((cp - 0xD800) << 10) + (lo - 0xDC00))
                    {
                        out.push(c);
                        i += 12;
                        continue;
                    }
                    out.push('\u{FFFD}');
                    i += 6;
                    continue;
                }
                match char::from_u32(cp) {
                    Some(c) => {
                        out.push(c);
                        i += 6;
                        continue;
                    }
                    // 低代理或非法码点
                    None => {
                        out.push('\u{FFFD}');
                        i += 6;
                        continue;
                    }
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_chinese() {
        assert_eq!(
            encode("你好，世界！"),
            "\\u4f60\\u597d\\uff0c\\u4e16\\u754c\\uff01"
        );
    }

    #[test]
    fn encode_ascii() {
        assert_eq!(encode("Hi"), "\\u0048\\u0069");
    }

    #[test]
    fn encode_emoji_uses_surrogate_pair() {
        assert_eq!(encode("😀"), "\\ud83d\\ude00");
    }

    #[test]
    fn decode_chinese() {
        assert_eq!(
            decode("\\u4f60\\u597d\\uff0c\\u4e16\\u754c\\uff01"),
            "你好，世界！"
        );
    }

    #[test]
    fn decode_surrogate_pair() {
        assert_eq!(decode("\\ud83d\\ude00"), "😀");
    }

    #[test]
    fn decode_keeps_plain_text() {
        assert_eq!(decode("a\\u0042c"), "aBc");
    }

    #[test]
    fn decode_invalid_escape_kept_as_is() {
        assert_eq!(decode("\\uZZZZ"), "\\uZZZZ");
    }

    #[test]
    fn round_trip() {
        let s = "DevTools 工具 😀";
        assert_eq!(decode(&encode(s)), s);
    }
}

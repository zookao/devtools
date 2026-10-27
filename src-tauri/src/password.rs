//! 密码生成：按字符类型从候选集随机取样（ThreadRng，CSPRNG 系统随机源）。

use std::collections::HashSet;

use rand::Rng;
use serde::Deserialize;

const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()-_=+[]{}|;:,.<>?/~`\"'\\";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PasswordOptions {
    pub length: usize,
    pub count: usize,
    pub upper: bool,
    pub lower: bool,
    pub digits: bool,
    pub symbols: bool,
    /// 每个密码至少包含每个已选类型的一个字符
    pub require_each: bool,
    /// 额外候选字符（不受类型开关限制）
    pub extra_chars: String,
    /// 从所有候选集中排除的字符
    pub exclude_chars: String,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 12,
            count: 10,
            upper: true,
            lower: true,
            digits: true,
            symbols: true,
            require_each: true,
            extra_chars: String::new(),
            exclude_chars: String::new(),
        }
    }
}

/// 0..n 的均匀随机整数
fn rand_below(n: usize) -> usize {
    rand::rng().random_range(0..n)
}

fn shuffle(chars: &mut [char]) {
    for i in (1..chars.len()).rev() {
        let j = rand_below(i + 1);
        chars.swap(i, j);
    }
}

pub fn generate(o: &PasswordOptions) -> Result<Vec<String>, String> {
    let length = o.length.clamp(4, 128);
    let count = o.count.clamp(1, 100);

    if !(o.upper || o.lower || o.digits || o.symbols) {
        return Err("请至少选择一种字符类型".to_string());
    }

    let exclude: HashSet<char> = o.exclude_chars.chars().collect();
    let filter = |s: &str| -> Vec<char> { s.chars().filter(|c| !exclude.contains(c)).collect() };

    let mut groups: Vec<Vec<char>> = Vec::new();
    if o.upper {
        groups.push(filter(UPPER));
    }
    if o.lower {
        groups.push(filter(LOWER));
    }
    if o.digits {
        groups.push(filter(DIGITS));
    }
    if o.symbols {
        groups.push(filter(SYMBOLS));
    }

    if groups.iter().any(|g| g.is_empty()) {
        return Err("排除字符过多，候选字符集为空".to_string());
    }

    let extras: Vec<char> = o
        .extra_chars
        .chars()
        .filter(|c| !c.is_whitespace() && !exclude.contains(c))
        .collect();

    if o.require_each && groups.len() > length {
        return Err(format!(
            "长度不足以包含所有已选类型（至少需要 {} 位）",
            groups.len()
        ));
    }

    let combined: Vec<char> = groups
        .iter()
        .flatten()
        .copied()
        .chain(extras.iter().copied())
        .collect();

    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let mut chars: Vec<char> = Vec::with_capacity(length);
        if o.require_each {
            for g in &groups {
                chars.push(g[rand_below(g.len())]);
            }
        }
        while chars.len() < length {
            chars.push(combined[rand_below(combined.len())]);
        }
        shuffle(&mut chars);
        out.push(chars.into_iter().collect());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> PasswordOptions {
        PasswordOptions::default()
    }

    #[test]
    fn respects_length_and_count() {
        let o = PasswordOptions {
            length: 20,
            count: 7,
            ..opts()
        };
        let out = generate(&o).unwrap();
        assert_eq!(out.len(), 7);
        assert!(out.iter().all(|p| p.chars().count() == 20));
    }

    #[test]
    fn only_digits_when_selected() {
        let o = PasswordOptions {
            upper: false,
            lower: false,
            symbols: false,
            digits: true,
            require_each: false,
            ..opts()
        };
        let out = generate(&o).unwrap();
        assert!(out.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())));
    }

    #[test]
    fn require_each_type_present() {
        let o = opts();
        let out = generate(&o).unwrap();
        for p in out {
            assert!(p.chars().any(|c| c.is_ascii_uppercase()), "{p}");
            assert!(p.chars().any(|c| c.is_ascii_lowercase()), "{p}");
            assert!(p.chars().any(|c| c.is_ascii_digit()), "{p}");
            assert!(p.chars().any(|c| !c.is_ascii_alphanumeric()), "{p}");
        }
    }

    #[test]
    fn excluded_chars_never_appear() {
        let o = PasswordOptions {
            exclude_chars: "Il1O0o".to_string(),
            length: 40,
            count: 20,
            ..opts()
        };
        let out = generate(&o).unwrap();
        for p in out {
            assert!(
                !p.chars().any(|c| "Il1O0o".contains(c)),
                "excluded char leaked: {p}"
            );
        }
    }

    #[test]
    fn extra_chars_used() {
        let o = PasswordOptions {
            upper: false,
            lower: false,
            digits: false,
            symbols: true,
            extra_chars: "€".to_string(),
            length: 64,
            count: 5,
            ..opts()
        };
        let out = generate(&o).unwrap();
        assert!(out.iter().any(|p| p.contains('€')));
    }

    #[test]
    fn no_type_selected_errors() {
        let o = PasswordOptions {
            upper: false,
            lower: false,
            digits: false,
            symbols: false,
            ..opts()
        };
        assert!(generate(&o).is_err());
    }
}

//! 容错预处理：在严格解析失败后按需尝试修复。
//!
//! 三个独立 pass，均基于单遍字符状态机（区分字符串内/外，处理转义），
//! 只在字符串字面量之外做修改，保证 `"a // b"`、`"a,}"` 等字符串内容不被破坏。
//! 删除注释时保留换行符，使原始行列号不漂移（错误位置始终按原文报告）。

use super::options::JsonOptions;

pub struct Preprocessed {
    pub text: String,
    pub changed: bool,
    pub notes: Vec<String>,
}

pub fn preprocess(input: &str, o: &JsonOptions) -> Preprocessed {
    let mut cur = input.to_string();
    let mut notes = Vec::new();
    if o.strip_control_chars {
        cur = strip_control_chars(&cur, &mut notes);
    }
    if o.allow_comments {
        cur = strip_comments(&cur, &mut notes);
    }
    if o.allow_trailing_commas {
        cur = strip_trailing_commas(&cur, &mut notes);
    }
    if o.auto_repair {
        cur = quote_bare_keys(&cur, &mut notes);
    }
    Preprocessed {
        changed: cur != input,
        text: cur,
        notes,
    }
}

#[derive(PartialEq, Clone, Copy)]
enum St {
    Normal,
    InStr,
    /// 字符串内的转义（下一个字符原样保留）
    Esc,
    /// 行注释 //
    Line,
    /// 块注释 /* ... */
    Block,
    /// 块注释中刚遇到 *（可能是 */
    BlockStar,
}

/// 去掉 // 与 /* */ 注释（仅字符串外）。换行符保留以维持行号。
fn strip_comments(s: &str, notes: &mut Vec<String>) -> String {
    let mut out = String::with_capacity(s.len());
    let mut st = St::Normal;
    let mut chars = s.chars().peekable();
    let mut removed = 0usize;

    while let Some(c) = chars.next() {
        match st {
            St::Normal => match c {
                '"' => {
                    out.push(c);
                    st = St::InStr;
                }
                '/' => match chars.peek() {
                    Some('/') => {
                        chars.next();
                        removed += 1;
                        st = St::Line;
                    }
                    Some('*') => {
                        chars.next();
                        removed += 1;
                        st = St::Block;
                    }
                    _ => out.push(c),
                },
                _ => out.push(c),
            },
            St::InStr => match c {
                '\\' => {
                    out.push(c);
                    st = St::Esc;
                }
                '"' => {
                    out.push(c);
                    st = St::Normal;
                }
                _ => out.push(c),
            },
            St::Esc => {
                out.push(c);
                st = St::InStr;
            }
            St::Line => {
                if c == '\n' {
                    out.push('\n');
                    st = St::Normal;
                }
            }
            St::Block => match c {
                '\n' => out.push('\n'),
                '*' => st = St::BlockStar,
                _ => {}
            },
            St::BlockStar => match c {
                '/' => st = St::Normal,
                '*' => {}
                '\n' => {
                    out.push('\n');
                    st = St::Block;
                }
                _ => st = St::Block,
            },
        }
    }
    if removed > 0 {
        notes.push(format!("Removed {} comment(s).", removed));
    }
    out
}

/// 去掉容器闭合前的尾随逗号（仅字符串外）。
/// 记录逗号位置，跳过空白后若下一个有效字符是 `}`/`]` 则连同其后空白一并删除。
fn strip_trailing_commas(s: &str, notes: &mut Vec<String>) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_str = false;
    let mut esc = false;
    let mut pending: Option<usize> = None; // out 中逗号的位置（push 之前）
    let mut removed = 0usize;

    for c in s.chars() {
        if in_str {
            out.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => {
                pending = None;
                out.push(c);
                in_str = true;
            }
            ',' => {
                pending = Some(out.len());
                out.push(c);
            }
            '}' | ']' => {
                if let Some(idx) = pending.take() {
                    out.truncate(idx);
                    removed += 1;
                }
                out.push(c);
            }
            c if c.is_whitespace() => out.push(c),
            _ => {
                pending = None;
                out.push(c);
            }
        }
    }
    if removed > 0 {
        notes.push(format!("Removed {} trailing comma(s).", removed));
    }
    out
}

/// 容器层级状态（只需区分“期望什么”，不需要 AfterValue：
/// `,` 统一转到期望下一个 key/元素，`}`/`]` 统一闭合）。
#[derive(PartialEq, Clone, Copy)]
enum Layer {
    /// 对象内：期望 key 开始（或 `}`）
    ObjExpectKey,
    /// 对象内：key 之后期望 `:`
    ObjExpectColon,
    /// 对象内：冒号后期望 value
    ObjExpectValue,
    /// 数组内：期望 value（或 `]`）
    ArrExpectValue,
}

fn is_bare_key_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$' || c == '-'
}

/// 给对象里缺失引号的裸键补引号：`{a: 1}` → `{"a": 1}`。
/// 仅在“期望 key”的位置处理，`true`/`null`/数字等值永不被改；数组内裸字符串不修（过于激进）。
fn quote_bare_keys(s: &str, notes: &mut Vec<String>) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut stack: Vec<Layer> = Vec::new();
    let mut in_str = false;
    let mut esc = false;
    let mut quoted = 0usize;

    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
                // 字符串 key 结束 → 期望冒号
                if stack.last() == Some(&Layer::ObjExpectKey) {
                    *stack.last_mut().unwrap() = Layer::ObjExpectColon;
                }
            }
            continue;
        }
        match c {
            '"' => {
                out.push(c);
                in_str = true;
            }
            '{' => {
                out.push(c);
                stack.push(Layer::ObjExpectKey);
            }
            '[' => {
                out.push(c);
                stack.push(Layer::ArrExpectValue);
            }
            '}' | ']' => {
                out.push(c);
                stack.pop();
            }
            ':' => {
                out.push(c);
                if stack.last() == Some(&Layer::ObjExpectColon) {
                    *stack.last_mut().unwrap() = Layer::ObjExpectValue;
                }
            }
            ',' => {
                out.push(c);
                if let Some(top) = stack.last_mut() {
                    match top {
                        Layer::ObjExpectKey
                        | Layer::ObjExpectColon
                        | Layer::ObjExpectValue => *top = Layer::ObjExpectKey,
                        Layer::ArrExpectValue => {}
                    }
                }
            }
            _ => {
                // 期望 key 的位置遇到裸标识符首字符 → 吞掉整个标识符并补引号
                if stack.last() == Some(&Layer::ObjExpectKey) && is_bare_key_char(c) {
                    let mut ident = String::new();
                    ident.push(c);
                    while let Some(&n) = chars.peek() {
                        if is_bare_key_char(n) {
                            ident.push(n);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    out.push('"');
                    out.push_str(&ident);
                    out.push('"');
                    *stack.last_mut().unwrap() = Layer::ObjExpectColon;
                    quoted += 1;
                } else {
                    out.push(c);
                }
            }
        }
    }
    if quoted > 0 {
        notes.push(format!("Quoted {} bare key(s).", quoted));
    }
    out
}

/// 清理不可见控制字符（U+0000–U+001F）：
/// - 字符串外：剔除非法控制字符（\t \n \r 是合法空白，保留）
/// - 字符串内（非转义态）：转义为 \t \n \r 或 \u00XX，保留数据且使 JSON 合法
fn strip_control_chars(s: &str, notes: &mut Vec<String>) -> String {
    use St::{Esc, InStr, Normal};
    let mut out = String::with_capacity(s.len());
    let mut st = Normal;
    let mut stripped = 0usize;
    let mut escaped = 0usize;

    for c in s.chars() {
        let is_ctrl = (c as u32) < 0x20;
        match st {
            Normal => {
                if is_ctrl && c != '\t' && c != '\n' && c != '\r' {
                    stripped += 1;
                } else {
                    out.push(c);
                    if c == '"' {
                        st = InStr;
                    }
                }
            }
            InStr => {
                if is_ctrl {
                    escaped += 1;
                    match c {
                        '\t' => out.push_str("\\t"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        _ => out.push_str(&format!("\\u{:04x}", c as u32)),
                    }
                } else {
                    out.push(c);
                    if c == '\\' {
                        st = Esc;
                    } else if c == '"' {
                        st = Normal;
                    }
                }
            }
            Esc => {
                out.push(c);
                st = InStr;
            }
            // 本 pass 独立运行，不会进入注释态；防御性保留原字符
            _ => out.push(c),
        }
    }

    if stripped + escaped > 0 {
        notes.push(format!(
            "Removed {stripped} and escaped {escaped} invisible control character(s)."
        ));
    }
    out
}

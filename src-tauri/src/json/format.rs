use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};
use serde_json::Value;

use super::options::Indent;

/// 序列化为指定缩进风格的 JSON 文本。非 Minify 时末尾补一个换行，便于阅读与复制。
pub fn render(v: &Value, indent: Indent) -> Result<String, serde_json::Error> {
    let mut buf = Vec::new();
    match indent {
        Indent::Minify => {
            v.serialize(&mut Serializer::new(&mut buf))?;
        }
        Indent::TwoSpaces | Indent::FourSpaces | Indent::Tab => {
            let pad: &[u8] = match indent {
                Indent::FourSpaces => b"    ",
                Indent::Tab => b"\t",
                _ => b"  ",
            };
            v.serialize(&mut Serializer::with_formatter(
                &mut buf,
                PrettyFormatter::with_indent(pad),
            ))?;
        }
    }
    let mut s = String::from_utf8(buf).expect("serde_json output is valid UTF-8");
    if !matches!(indent, Indent::Minify) {
        s.push('\n');
    }
    Ok(s)
}

/// 递归排序所有对象的 key（数组内的对象同样递归）。preserve_order 下 Map 保序，重排后即为排序结果。
pub fn sort_keys(v: &mut Value) {
    match v {
        Value::Object(m) => {
            let mut entries: Vec<(String, Value)> = std::mem::take(m).into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            m.extend(entries);
            for (_, val) in m.iter_mut() {
                sort_keys(val);
            }
        }
        Value::Array(a) => a.iter_mut().for_each(sort_keys),
        _ => {}
    }
}

pub fn count_keys(v: &Value) -> usize {
    match v {
        Value::Object(m) => {
            m.len() + m.values().map(count_keys).sum::<usize>()
        }
        Value::Array(a) => a.iter().map(count_keys).sum(),
        _ => 0,
    }
}

pub fn depth(v: &Value) -> usize {
    match v {
        Value::Object(m) => 1 + m.values().map(depth).max().unwrap_or(0),
        Value::Array(a) => 1 + a.iter().map(depth).max().unwrap_or(0),
        _ => 0,
    }
}

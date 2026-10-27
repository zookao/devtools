//! JSON ↔ YAML 互转。
//!
//! JSON → YAML：自行逐节点输出，统一 2 空格缩进，数组元素额外缩进一层
//! （`ports:` 下的 `- 80` 缩进 4 空格），保持 key 顺序与输入一致。
//! YAML → JSON：用 serde_yaml 解析后输出 2 空格缩进的 JSON。

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

pub fn json_to_yaml(input: &str) -> Result<String, String> {
    let json: JsonValue =
        serde_json::from_str(input).map_err(|e| format!("JSON解析失败: {e}"))?;

    let mut lines: Vec<String> = Vec::new();
    emit(&json, 0, &mut lines);
    let mut out = lines.join("\n");
    out.push('\n');
    Ok(out)
}

pub fn yaml_to_json(input: &str) -> Result<String, String> {
    let yaml: YamlValue =
        serde_yaml::from_str(input).map_err(|e| format!("YAML解析失败: {e}"))?;
    let json = yaml_to_json_value(&yaml)?;
    serde_json::to_string_pretty(&json).map_err(|e| format!("JSON生成失败: {e}"))
}

// ---------- JSON → YAML 输出 ----------

/// 逐节点输出 YAML 文本行
fn emit(value: &JsonValue, indent: usize, out: &mut Vec<String>) {
    let pad = " ".repeat(indent);
    match value {
        JsonValue::Object(map) => {
            if map.is_empty() {
                out.push(format!("{pad}{{}}"));
                return;
            }
            for (key, val) in map {
                let key = quote_string(key);
                match val {
                    JsonValue::Object(m) if m.is_empty() => out.push(format!("{pad}{key}: {{}}")),
                    JsonValue::Array(a) if a.is_empty() => out.push(format!("{pad}{key}: []")),
                    JsonValue::Object(_) | JsonValue::Array(_) => {
                        out.push(format!("{pad}{key}:"));
                        emit(val, indent + 2, out);
                    }
                    _ => out.push(format!("{pad}{key}: {}", scalar(val))),
                }
            }
        }
        JsonValue::Array(items) => {
            if items.is_empty() {
                out.push(format!("{pad}[]"));
                return;
            }
            for item in items {
                match item {
                    JsonValue::Object(m) if m.is_empty() => out.push(format!("{pad}- {{}}")),
                    JsonValue::Array(a) if a.is_empty() => out.push(format!("{pad}- []")),
                    JsonValue::Object(_) | JsonValue::Array(_) => {
                        // 首行并入 "- "，其余行保持子级缩进
                        let mut sub: Vec<String> = Vec::new();
                        emit(item, indent + 2, &mut sub);
                        let first = sub.remove(0);
                        let rest = &first[indent + 2..];
                        out.push(format!("{pad}- {rest}"));
                        out.extend(sub);
                    }
                    _ => out.push(format!("{pad}- {}", scalar(item))),
                }
            }
        }
        _ => out.push(format!("{pad}{}", scalar(value))),
    }
}

fn scalar(v: &JsonValue) -> String {
    match v {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => quote_string(s),
        JsonValue::Array(_) | JsonValue::Object(_) => {
            unreachable!("容器类型由 emit 处理")
        }
    }
}

/// 按 YAML 语法给字符串按需加引号
fn quote_string(s: &str) -> String {
    if !needs_quotes(s) {
        return s.to_string();
    }
    if !s.contains('\'') && !s.contains('\n') && !s.contains('\t') && !s.contains('\r') {
        return format!("'{s}'");
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn needs_quotes(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    let first = s.chars().next().unwrap();
    if first == ' ' || first == '\t' {
        return true;
    }
    if s.ends_with(' ') || s.ends_with('\t') {
        return true;
    }
    if s.contains('\n') || s.contains('\t') || s.contains('\r') {
        return true;
    }
    if s.contains(": ") || s.contains(" #") {
        return true;
    }
    if "#&*!|>'\"%@`".contains(first) {
        return true;
    }
    if matches!(first, '[' | ']' | '{' | '}' | ',') {
        return true;
    }
    // "-" "?" ":" 单独出现或后接空格时才需要引号（-foo 是合法裸串）
    if matches!(first, '-' | '?' | ':') && (s.len() == 1 || s.as_bytes()[1] == b' ') {
        return true;
    }
    is_ambiguous(s)
}

/// 是否会被解析成非字符串（布尔/空值/数字/特殊浮点）
fn is_ambiguous(s: &str) -> bool {
    const WORDS: [&str; 12] = [
        "null", "~", "true", "false", "yes", "no", "on", "off", "y", "n", ".inf", ".nan",
    ];
    let lower = s.to_ascii_lowercase();
    if WORDS.contains(&lower.as_str()) {
        return true;
    }
    if lower.starts_with("-.") || lower.starts_with("+.") {
        let rest = &lower[1..];
        if rest == ".inf" || rest == ".nan" {
            return true;
        }
    }
    s.parse::<i64>().is_ok() || s.parse::<u64>().is_ok() || s.parse::<f64>().is_ok()
}

// ---------- YAML → JSON 解析 ----------

fn yaml_to_json_value(v: &YamlValue) -> Result<JsonValue, String> {
    Ok(match v {
        YamlValue::Null => JsonValue::Null,
        YamlValue::Bool(b) => JsonValue::Bool(*b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::from(i)
            } else if let Some(u) = n.as_u64() {
                JsonValue::from(u)
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(JsonValue::Number)
                    .ok_or_else(|| "数字超出 JSON 可表示范围".to_string())?
            } else {
                return Err("不支持的数字格式".to_string());
            }
        }
        YamlValue::String(s) => JsonValue::String(s.clone()),
        YamlValue::Sequence(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(yaml_to_json_value(item)?);
            }
            JsonValue::Array(out)
        }
        YamlValue::Mapping(map) => {
            let mut out = serde_json::Map::new();
            for (k, val) in map {
                out.insert(yaml_key_to_string(k)?, yaml_to_json_value(val)?);
            }
            JsonValue::Object(out)
        }
        YamlValue::Tagged(tagged) => yaml_to_json_value(&tagged.value)?,
    })
}

/// JSON 的 key 必须是字符串：YAML 里的数字/布尔/null 键按下标量文本转换。
fn yaml_key_to_string(k: &YamlValue) -> Result<String, String> {
    Ok(match k {
        YamlValue::String(s) => s.clone(),
        YamlValue::Bool(b) => b.to_string(),
        YamlValue::Number(n) => n.to_string(),
        YamlValue::Null => "null".to_string(),
        _ => return Err("不支持的映射键类型：只允许字符串、数字、布尔或 null".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_object_to_yaml() {
        let out = json_to_yaml(r#"{"name":"devtools","port":8080,"debug":true}"#).unwrap();
        assert_eq!(out, "name: devtools\nport: 8080\ndebug: true\n");
    }

    #[test]
    fn indents_nested_objects_and_lists() {
        let out =
            json_to_yaml(r#"{"server":{"host":"127.0.0.1","ports":[80,443]},"tags":["a","b"]}"#)
                .unwrap();
        assert_eq!(
            out,
            "server:\n  host: 127.0.0.1\n  ports:\n    - 80\n    - 443\ntags:\n  - a\n  - b\n"
        );
    }

    #[test]
    fn keeps_key_order() {
        let out = json_to_yaml(r#"{"z":1,"a":2,"m":3}"#).unwrap();
        assert_eq!(out, "z: 1\na: 2\nm: 3\n");
    }

    #[test]
    fn quotes_ambiguous_strings() {
        let out = json_to_yaml(r#"{"a":"true","empty":"","num":"8080"}"#).unwrap();
        assert_eq!(out, "a: 'true'\nempty: ''\nnum: '8080'\n");
    }

    #[test]
    fn array_of_objects_inlines_first_key() {
        let out = json_to_yaml(r#"{"list":[{"id":1,"name":"a"},{"id":2,"name":"b"}]}"#).unwrap();
        assert_eq!(
            out,
            "list:\n  - id: 1\n    name: a\n  - id: 2\n    name: b\n"
        );
    }

    #[test]
    fn nested_arrays_indent_per_level() {
        let out = json_to_yaml(r#"{"m":[[1,2],[]]}"#).unwrap();
        assert_eq!(out, "m:\n  - - 1\n    - 2\n  - []\n");
    }

    #[test]
    fn empty_containers_are_inline() {
        let out = json_to_yaml(r#"{"a":{},"b":[]}"#).unwrap();
        assert_eq!(out, "a: {}\nb: []\n");
    }

    #[test]
    fn converts_yaml_to_json() {
        let out = yaml_to_json("name: devtools\nport: 8080\nenabled: false\n").unwrap();
        assert_eq!(
            out,
            "{\n  \"name\": \"devtools\",\n  \"port\": 8080,\n  \"enabled\": false\n}"
        );
    }

    #[test]
    fn yaml_flow_style_and_nested_list() {
        let out = yaml_to_json("- a\n- b: 1\n  c: [1, 2]\n").unwrap();
        assert_eq!(
            out,
            "[\n  \"a\",\n  {\n    \"b\": 1,\n    \"c\": [\n      1,\n      2\n    ]\n  }\n]"
        );
    }

    #[test]
    fn yaml_null_and_tagged_values() {
        let out = yaml_to_json("a: null\nb: !!str 123\n").unwrap();
        assert_eq!(out, "{\n  \"a\": null,\n  \"b\": \"123\"\n}");
    }

    #[test]
    fn numeric_yaml_keys_become_strings() {
        let out = yaml_to_json("1: one\n2: two\n").unwrap();
        assert_eq!(out, "{\n  \"1\": \"one\",\n  \"2\": \"two\"\n}");
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(json_to_yaml("{a:1}").is_err());
    }

    #[test]
    fn rejects_invalid_yaml() {
        assert!(yaml_to_json("a: [1, 2").is_err());
    }

    #[test]
    fn round_trip() {
        let json = r#"{"a":[1,2],"b":{"c":"中文"},"d":null,"e":[{"f":1}],"g":[]}"#;
        let yaml = json_to_yaml(json).unwrap();
        let back = yaml_to_json(&yaml).unwrap();
        let orig: JsonValue = serde_json::from_str(json).unwrap();
        let parsed: JsonValue = serde_json::from_str(&back).unwrap();
        assert_eq!(orig, parsed);
    }
}

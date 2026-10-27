//! JSON → Markdown 表格（参数说明文档）。
//!
//! 移植自 Go 项目 json2md/converter：支持 JSON 中的 `//` 行注释，
//! 注释形如 `//必填，发票id`、`//非必填，备注`，用于填充「是否必填」「参数说明」两列。

use std::collections::HashMap;

use serde_json::Value;

struct FieldInfo {
    name: String,
    ty: String,
    required: bool,
    desc: String,
}

#[derive(Clone)]
struct CommentInfo {
    required: bool,
    desc: String,
}

/// JSON 字符串 → Markdown 表格文本
pub fn convert(input: &str) -> Result<String, String> {
    let comments = extract_comments(input);
    let clean = remove_comments(input);

    let root: Value =
        serde_json::from_str(&clean).map_err(|e| format!("JSON解析失败: {e}"))?;

    let obj = match &root {
        Value::Object(map) => map,
        _ => return Err("JSON解析失败: 顶层必须是 JSON 对象".to_string()),
    };

    let mut fields: Vec<FieldInfo> = Vec::new();
    for (key, value) in obj {
        let (required, desc, label) = child_meta(key, &comments, None);
        let name = key.clone();
        parse_value(
            value,
            &name,
            required,
            &desc,
            label.as_deref(),
            &comments,
            &mut fields,
        );
    }

    let mut out = String::new();
    out.push_str("| 参数名 | 类型 | 是否必填 | 参数说明 |\n");
    out.push_str("|--------|------|----------|----------|\n");
    for f in fields {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            f.name,
            f.ty,
            bool_to_required(f.required),
            f.desc
        ));
    }
    Ok(out)
}

/// 递归解析一个值，展开对象/数组，叶子节点写入 fields
///
/// `label` 是本节点的备注（仅当本节点有自己的 `//` 注释时存在），
/// 展开子字段时会作为说明前缀，形如 `机构信息：organ_id`。
fn parse_value(
    value: &Value,
    prefix: &str,
    required: bool,
    comment: &str,
    label: Option<&str>,
    comments: &HashMap<String, CommentInfo>,
    fields: &mut Vec<FieldInfo>,
) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let name = format!("{prefix}[\"{key}\"]");
                let (req, desc, lbl) = child_meta(key, comments, label);
                parse_value(val, &name, req, &desc, lbl.as_deref(), comments, fields);
            }
        }
        Value::Array(items) => {
            if let Some(first) = items.first() {
                // 取第一个元素作为示例
                match first {
                    Value::Object(map) => {
                        for (key, val) in map {
                            let name = format!("{prefix}[0][\"{key}\"]");
                            let (req, desc, lbl) = child_meta(key, comments, label);
                            parse_value(val, &name, req, &desc, lbl.as_deref(), comments, fields);
                        }
                    }
                    other => fields.push(FieldInfo {
                        name: format!("{prefix}[0]"),
                        ty: get_type(other),
                        required,
                        desc: comment.to_string(),
                    }),
                }
            } else {
                fields.push(FieldInfo {
                    name: prefix.to_string(),
                    ty: "array".to_string(),
                    required,
                    desc: comment.to_string(),
                });
            }
        }
        other => fields.push(FieldInfo {
            name: prefix.to_string(),
            ty: get_type(other),
            required,
            desc: comment.to_string(),
        }),
    }
}

fn get_type(value: &Value) -> String {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                "int"
            } else {
                let f = n.as_f64().unwrap_or(0.0);
                // 与 Go 版一致：小数部分为 0 的无精度值也视为 int
                if f.fract() == 0.0 && f.abs() <= 9.2e18 {
                    "int"
                } else {
                    "float"
                }
            }
        }
    }
    .to_string()
}

fn bool_to_required(required: bool) -> &'static str {
    if required {
        "必填"
    } else {
        "非必填"
    }
}

/// 计算一个字段的三元信息：(是否必填, 说明, 供子字段继承的备注)
///
/// - 自己带 `//` 注释：用注释里的必填与描述；
/// - 没有注释但有父级备注：说明写成 `父备注：key`，并继续向下传递；
/// - 都没有：说明回退为 key 名。
fn child_meta(
    key: &str,
    comments: &HashMap<String, CommentInfo>,
    parent_label: Option<&str>,
) -> (bool, String, Option<String>) {
    if let Some(info) = comments.get(key) {
        return (info.required, info.desc.clone(), Some(info.desc.clone()));
    }
    match parent_label {
        Some(label) => {
            let desc = format!("{label}：{key}");
            (true, desc.clone(), Some(desc))
        }
        None => (true, key.to_string(), None),
    }
}

// ---------- 注释提取 ----------

fn extract_comments(input: &str) -> HashMap<String, CommentInfo> {
    let mut comments: HashMap<String, CommentInfo> = HashMap::new();
    let mut current_key: Option<String> = None;
    let mut in_multiline = false;
    let mut brace_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;

    for line in input.lines() {
        let trimmed = line.trim();

        if !in_multiline {
            if let Some((key, rest)) = split_key_line(trimmed) {
                if is_value_complete(rest) {
                    let comment = extract_line_comment(rest);
                    if !comment.is_empty() {
                        comments.insert(key, parse_comment(&comment));
                    }
                } else {
                    // 值跨行（对象或数组开始）
                    current_key = Some(key);
                    in_multiline = true;
                    brace_depth = count_char(rest, '{') - count_char(rest, '}');
                    bracket_depth = count_char(rest, '[') - count_char(rest, ']');
                }
                continue;
            }
        }

        if in_multiline {
            brace_depth += count_char(trimmed, '{') - count_char(trimmed, '}');
            bracket_depth += count_char(trimmed, '[') - count_char(trimmed, ']');

            if brace_depth <= 0 && bracket_depth <= 0 {
                in_multiline = false;
                brace_depth = 0;
                bracket_depth = 0;
                let comment = extract_line_comment(trimmed);
                if !comment.is_empty() {
                    if let Some(key) = current_key.take() {
                        comments.insert(key, parse_comment(&comment));
                    }
                }
                current_key = None;
            }
        }
    }

    comments
}

fn count_char(s: &str, c: char) -> i32 {
    s.chars().filter(|&x| x == c).count() as i32
}

/// 匹配 `^"key"\s*:\s*rest$`，返回 (key, rest)
fn split_key_line(line: &str) -> Option<(String, &str)> {
    let bytes = line.as_bytes();
    if bytes.first() != Some(&b'"') {
        return None;
    }

    let mut i = 1;
    while i < bytes.len() && bytes[i] != b'"' {
        i += 1;
    }
    if i >= bytes.len() || i == 1 {
        return None;
    }
    let key = line[1..i].to_string();

    let mut j = i + 1;
    while j < bytes.len() && (bytes[j] as char).is_whitespace() {
        j += 1;
    }
    if bytes.get(j) != Some(&b':') {
        return None;
    }
    j += 1;
    while j < bytes.len() && (bytes[j] as char).is_whitespace() {
        j += 1;
    }
    Some((key, &line[j..]))
}

/// 判断值是否在同一行内完成
fn is_value_complete(rest: &str) -> bool {
    let rest = rest.trim();
    if rest.is_empty() {
        return false;
    }
    if !rest.starts_with('{') && !rest.starts_with('[') {
        return true;
    }

    let mut depth: i32 = 0;
    let mut in_str = false;
    let mut escaped = false;
    for ch in rest.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_str {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_str = !in_str;
            continue;
        }
        if !in_str {
            if ch == '{' || ch == '[' {
                depth += 1;
            } else if ch == '}' || ch == ']' {
                depth -= 1;
            }
        }
    }
    depth == 0
}

/// `//` 的位置（忽略字符串内部的 `//`）
fn comment_start(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut in_string = false;
    let mut escaped = false;
    for (i, ch) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string && ch == '/' && bytes.get(i + 1) == Some(&b'/') {
            return Some(i);
        }
    }
    None
}

fn extract_line_comment(line: &str) -> String {
    match comment_start(line) {
        Some(i) => line[i + 2..].trim().to_string(),
        None => String::new(),
    }
}

fn remove_comments(input: &str) -> String {
    input
        .lines()
        .map(|line| match comment_start(line) {
            Some(i) => line[..i].trim_end_matches(|c| c == ' ' || c == '\t').to_string(),
            None => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_comment(comment: &str) -> CommentInfo {
    let required = !(comment.contains("非必填")
        || comment.contains("选填")
        || comment.contains("可选"));

    let mut desc = comment;
    const MARKERS: [&str; 8] = [
        "必填，", "必填,", "非必填，", "非必填,", "选填，", "选填,", "可选，", "可选,",
    ];
    for marker in MARKERS {
        if let Some(rest) = desc.strip_prefix(marker) {
            desc = rest;
            break;
        }
    }
    let desc = desc.trim();

    CommentInfo {
        required,
        desc: if desc.is_empty() {
            comment.to_string()
        } else {
            desc.to_string()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_flat_object_with_comments() {
        let input = r#"{
  "id": 12, //必填，发票id
  "invoice_date": "20260310", //必填，开票日期
  "actual_amount": "40000", //非必填，实际开票金额
  "remark": "x"
}"#;
        let out = convert(input).unwrap();
        assert!(out.starts_with("| 参数名 | 类型 | 是否必填 | 参数说明 |\n|--------|------|----------|----------|\n"));
        assert!(out.contains("| id | int | 必填 | 发票id |\n"));
        assert!(out.contains("| invoice_date | string | 必填 | 开票日期 |\n"));
        assert!(out.contains("| actual_amount | string | 非必填 | 实际开票金额 |\n"));
        // 无注释时，说明列回退为字段名
        assert!(out.contains("| remark | string | 必填 | remark |\n"));
    }

    #[test]
    fn expands_array_of_objects() {
        let input = r#"{
  "invoice_file": [
    {
      "file_name": "a.pdf",
      "file_url": "https://x/a.pdf"
    }
  ] //必填，附件
}"#;
        let out = convert(input).unwrap();
        assert!(out.contains("| invoice_file[0][\"file_name\"] | string | 必填 | 附件：file_name |\n"));
        assert!(out.contains("| invoice_file[0][\"file_url\"] | string | 必填 | 附件：file_url |\n"));
    }

    #[test]
    fn array_of_primitives_uses_first_element() {
        let input = r#"{
  "ids": [1, 2, 3] //必填，编号列表
}"#;
        let out = convert(input).unwrap();
        assert!(out.contains("| ids[0] | int | 必填 | 编号列表 |\n"));
    }

    #[test]
    fn empty_array_keeps_array_type() {
        let input = r#"{
  "tags": [] //选填，标签
}"#;
        let out = convert(input).unwrap();
        assert!(out.contains("| tags | array | 非必填 | 标签 |\n"));
    }

    #[test]
    fn nested_object_paths_and_multiline_comment() {
        let input = r#"{
  "organ": {
    "organ_id": 1,
    "organ_name": "测试机构"
  } //必填，机构信息
}"#;
        let out = convert(input).unwrap();
        assert!(out.contains("| organ[\"organ_id\"] | int | 必填 | 机构信息：organ_id |\n"));
        assert!(out.contains("| organ[\"organ_name\"] | string | 必填 | 机构信息：organ_name |\n"));
    }

    #[test]
    fn double_slash_inside_string_is_kept() {
        let input = r#"{
  "url": "https://example.com/a" //必填，地址
}"#;
        let out = convert(input).unwrap();
        assert!(out.contains("| url | string | 必填 | 地址 |\n"));
    }

    #[test]
    fn float_and_bool_and_null_types() {
        let input = r#"{
  "ratio": 1.25,
  "whole": 2.0,
  "enabled": true,
  "extra": null
}"#;
        let out = convert(input).unwrap();
        assert!(out.contains("| ratio | float |"));
        assert!(out.contains("| whole | int |"));
        assert!(out.contains("| enabled | boolean |"));
        assert!(out.contains("| extra | null |"));
    }

    #[test]
    fn converts_full_sample_like_go_version() {
        let input = r#"{
  "id": 12, //必填，发票id
  "invoice_date": "20260310", //必填，开票日期
  "invoice_number": "11111111111111", //必填，发票号
  "actual_amount": "40000", //非必填，实际开票金额
  "invoice_file": [
    {
      "file_name": "1-20240614 (2) (1).pdf",
      "file_url": "/pdf/pdf_1665.pdf",
      "file_url_format": "https://example.com/pdf.pdf?Expires=6698467986"
    },
    {
      "file_name": "1-20240614 (3).pdf",
      "file_url": "/pdf/pdf_1769.pdf",
      "file_url_format": "https://example.com/pdf2.pdf?Expires=6698467987"
    }
  ], //必填，附件
  "organ": {
    "organ_id": 1,
    "organ_name": "测试机构"
  } //必填，机构信息
}"#;
        let out = convert(input).unwrap();
        let expected = "| 参数名 | 类型 | 是否必填 | 参数说明 |
|--------|------|----------|----------|
| id | int | 必填 | 发票id |
| invoice_date | string | 必填 | 开票日期 |
| invoice_number | string | 必填 | 发票号 |
| actual_amount | string | 非必填 | 实际开票金额 |
| invoice_file[0][\"file_name\"] | string | 必填 | 附件：file_name |
| invoice_file[0][\"file_url\"] | string | 必填 | 附件：file_url |
| invoice_file[0][\"file_url_format\"] | string | 必填 | 附件：file_url_format |
| organ[\"organ_id\"] | int | 必填 | 机构信息：organ_id |
| organ[\"organ_name\"] | string | 必填 | 机构信息：organ_name |
";
        assert_eq!(out, expected);
    }

    #[test]
    fn rejects_non_object_root() {
        assert!(convert("[1, 2]").is_err());
        assert!(convert("{ bad json }").is_err());
    }

    #[test]
    fn parse_comment_extracts_required_and_desc() {
        let c = parse_comment("非必填,备注信息");
        assert!(!c.required);
        assert_eq!(c.desc, "备注信息");

        let c = parse_comment("可选：说明");
        assert!(!c.required);
        assert_eq!(c.desc, "可选：说明");
    }
}

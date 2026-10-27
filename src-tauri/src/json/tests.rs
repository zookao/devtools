use super::options::{Indent, JsonOptions};
use super::process;

fn opt_comments() -> JsonOptions {
    JsonOptions {
        allow_comments: true,
        ..Default::default()
    }
}

fn opt_trailing() -> JsonOptions {
    JsonOptions {
        allow_trailing_commas: true,
        ..Default::default()
    }
}

fn opt_repair() -> JsonOptions {
    JsonOptions {
        auto_repair: true,
        ..Default::default()
    }
}

// ---------- 合法输入 ----------

#[test]
fn valid_nested() {
    let o = process(r#"{"a":[1,2,{"b":null}],"c":true}"#, &Default::default());
    assert!(o.ok);
    assert_eq!(
        o.output.unwrap().trim_end(),
        "{\n  \"a\": [\n    1,\n    2,\n    {\n      \"b\": null\n    }\n  ],\n  \"c\": true\n}"
    );
}

#[test]
fn valid_chinese_keys_and_values() {
    let o = process(r#"{"名字":"张三","年龄":18}"#, &Default::default());
    assert!(o.ok);
    let out = o.output.unwrap();
    assert!(out.contains("\"名字\": \"张三\""));
    assert!(out.contains("\"年龄\": 18"));
}

#[test]
fn valid_escapes_and_unicode() {
    let o = process(r#"{"s":"a\"b\\c\u4e2d"}"#, &Default::default());
    assert!(o.ok);
}

#[test]
fn valid_top_level_scalars() {
    assert!(process("42", &Default::default()).ok);
    assert!(process("\"hi\"", &Default::default()).ok);
    assert!(process(" true ", &Default::default()).ok);
    assert!(process("[]", &Default::default()).ok);
    assert!(process("{}", &Default::default()).ok);
}

#[test]
fn valid_numbers() {
    assert!(process("[1e10, -3.5, 0, 1E-2]", &Default::default()).ok);
}

// ---------- preserve_order：key 顺序 ----------

#[test]
fn preserves_key_order_without_sort() {
    let o = process(r#"{"b":1,"a":2}"#, &Default::default());
    assert!(o.ok);
    let out = o.output.unwrap();
    assert!(out.find("\"b\"").unwrap() < out.find("\"a\"").unwrap(), "got: {out}");
}

#[test]
fn sorts_keys_when_enabled() {
    let o = process(
        r#"{"b":1,"a":{"z":1,"y":2}}"#,
        &JsonOptions {
            sort_keys: true,
            ..Default::default()
        },
    );
    assert!(o.ok);
    let out = o.output.unwrap();
    assert!(out.find("\"a\"").unwrap() < out.find("\"b\"").unwrap());
    assert!(out.find("\"y\"").unwrap() < out.find("\"z\"").unwrap());
}

// ---------- 缩进 ----------

#[test]
fn indent_variants() {
    let src = r#"{"a":1}"#;
    let two = process(src, &Default::default()).output.unwrap();
    assert!(two.contains("\n  \"a\": 1"), "two: {two:?}");

    let four = process(
        src,
        &JsonOptions {
            indent: Indent::FourSpaces,
            ..Default::default()
        },
    )
    .output
    .unwrap();
    assert!(four.contains("\n    \"a\": 1"), "four: {four:?}");

    let tab = process(
        src,
        &JsonOptions {
            indent: Indent::Tab,
            ..Default::default()
        },
    )
    .output
    .unwrap();
    assert!(tab.contains("\n\t\"a\": 1"), "tab: {tab:?}");

    let min = process(
        src,
        &JsonOptions {
            indent: Indent::Minify,
            ..Default::default()
        },
    )
    .output
    .unwrap();
    assert_eq!(min, "{\"a\":1}");
}

// ---------- 错误 ----------

#[test]
fn error_line_column() {
    let o = process("{\n  \"a\": 1,\n}", &Default::default());
    assert!(!o.ok);
    let e = o.error.unwrap();
    assert_eq!(e.line, 3);
    assert!(e.message.contains("trailing comma") || e.column > 0);
}

#[test]
fn error_empty_input() {
    let o = process("   \n  ", &Default::default());
    assert!(!o.ok && o.empty && o.error.is_none());
}

#[test]
fn error_trailing_garbage() {
    let o = process("{\"a\":1} xyz", &Default::default());
    assert!(!o.ok);
    assert!(o.error.unwrap().message.contains("trailing characters"));
}

#[test]
fn error_nan() {
    assert!(!process("NaN", &Default::default()).ok);
    assert!(!process("[Infinity]", &Default::default()).ok);
}

#[test]
fn error_bare_key_without_repair() {
    let o = process("{a: 1}", &Default::default());
    assert!(!o.ok && o.error.is_some());
}

// ---------- 修复：注释 ----------

#[test]
fn repair_line_comment() {
    let o = process("{\n  // comment\n  \"a\": 1\n}", &opt_comments());
    assert!(o.ok);
    assert!(o.repaired);
    let out = o.output.unwrap();
    assert!(out.contains("\"a\": 1"));
}

#[test]
fn repair_block_comment_multiline_keeps_lines() {
    let o = process("{\n  /* multi\n     line */\n  \"a\": 1\n}", &opt_comments());
    assert!(o.ok);
    assert!(o.repaired);
}

#[test]
fn repair_url_in_string_not_comment() {
    let o = process(r#"{"url":"https://example.com"}"#, &opt_comments());
    assert!(o.ok);
    assert!(!o.repaired, "合法输入不应被标记为 repaired");
    assert!(o.output.unwrap().contains("https://example.com"));
}

#[test]
fn repair_comment_inside_string_preserved() {
    let o = process(r#"{"s":"a // not comment"}"#, &opt_comments());
    assert!(o.ok && !o.repaired);
    assert!(o.output.unwrap().contains("a // not comment"));
}

// ---------- 修复：尾随逗号 ----------

#[test]
fn repair_trailing_comma_object() {
    let o = process("{\"a\":1,}", &opt_trailing());
    assert!(o.ok && o.repaired);
    assert_eq!(o.output.unwrap().trim(), "{\n  \"a\": 1\n}");
}

#[test]
fn repair_trailing_comma_array() {
    let o = process("[1,2, ]", &opt_trailing());
    assert!(o.ok && o.repaired);
}

#[test]
fn repair_trailing_comma_nested() {
    let o = process("{\"a\":[1,2,],}", &opt_trailing());
    assert!(o.ok && o.repaired);
}

#[test]
fn comma_inside_string_preserved() {
    let o = process(r#"{"s":"a,}"}}"#, &Default::default());
    // 本例整体非法（多了 }}），但不允许 trailing 修复破坏字符串
    assert!(!o.ok);
    let o2 = process(r#"{"s":"a,}"}"#, &opt_trailing());
    assert!(o2.ok && !o2.repaired);
}

#[test]
fn number_with_thousands_separator_not_repaired() {
    let o = process("[1,000]", &opt_trailing());
    assert!(!o.ok, "1,000 不应被误修复为 1000");
}

// ---------- 修复：裸键 ----------

#[test]
fn repair_bare_key() {
    let o = process("{a: 1}", &opt_repair());
    assert!(o.ok && o.repaired);
    assert_eq!(o.output.unwrap().trim(), "{\n  \"a\": 1\n}");
}

#[test]
fn repair_bare_keys_multiple_and_nested() {
    let o = process("{a: 1, b_2: true, nested: {x: null}}", &opt_repair());
    assert!(o.ok && o.repaired);
    let out = o.output.unwrap();
    assert!(out.contains("\"a\": 1"));
    assert!(out.contains("\"b_2\": true"));
    assert!(out.contains("\"x\": null"));
}

#[test]
fn repair_bare_key_with_hyphen_and_dollar() {
    let o = process("{content-type: \"x\", $ref: 1}", &opt_repair());
    assert!(o.ok && o.repaired);
}

#[test]
fn repair_bare_key_chinese() {
    let o = process("{名字: \"张三\"}", &opt_repair());
    assert!(o.ok && o.repaired);
    assert!(o.output.unwrap().contains("\"名字\": \"张三\""));
}

#[test]
fn repair_bare_key_values_not_touched() {
    // 值位置的 true/null/数字不能被加引号
    let o = process("[true, null, 1, \"a\"]", &opt_repair());
    assert!(o.ok && !o.repaired);
    assert_eq!(o.output.unwrap().trim(), "[\n  true,\n  null,\n  1,\n  \"a\"\n]");
}

#[test]
fn repair_bare_key_empty_object_array() {
    let o = process("{a:{},b:[]}", &opt_repair());
    assert!(o.ok && o.repaired);
}

// ---------- 修复组合与幂等 ----------

#[test]
fn repair_all_options_combined() {
    let src = "{\n  // note\n  a: 1, // trailing\n  b: [1,2,],\n}";
    let o = process(
        src,
        &JsonOptions {
            allow_comments: true,
            allow_trailing_commas: true,
            auto_repair: true,
            ..Default::default()
        },
    );
    assert!(o.ok && o.repaired);
    let out = o.output.unwrap();
    assert!(out.contains("\"a\": 1"));
    assert!(out.contains("\"b\": ["));
}

#[test]
fn repair_is_idempotent() {
    use super::repair::preprocess;
    let o = JsonOptions {
        allow_comments: true,
        allow_trailing_commas: true,
        auto_repair: true,
        ..Default::default()
    };
    let src = "{ // c\n a: 1, b: [1,],}";
    let p1 = preprocess(src, &o);
    let p2 = preprocess(&p1.text, &o);
    assert_eq!(p1.text, p2.text, "二次预处理应无变化");
}

#[test]
fn repair_failure_reports_original_error() {
    // 无法修复的输入：仍报原始行列
    let o = process("{a: 1", &opt_repair());
    assert!(!o.ok);
    let e = o.error.unwrap();
    assert!(e.line >= 1);
}

// ---------- 统计 ----------

#[test]
fn stats_counts() {
    let o = process(r#"{"a":{"b":1},"c":[1,2]}"#, &Default::default());
    assert!(o.ok);
    let s = o.stats.unwrap();
    assert_eq!(s.keys, 3); // a,b,c
    assert_eq!(s.depth, 2);
    assert!(s.bytes_in > 0 && s.bytes_out > 0);
}

#[test]
fn bom_stripped() {
    let o = process("\u{feff}{\"a\":1}", &Default::default());
    assert!(o.ok);
}

fn opt_strip() -> JsonOptions {
    JsonOptions {
        strip_control_chars: true,
        ..Default::default()
    }
}

// ---------- 控制字符清理 ----------

#[test]
fn strip_escapes_control_chars_inside_strings() {
    // 字符串内的 \x01 转义为 \u0001，数据保留
    let o = process("{\u{1}\"name\": \"a\u{1}b\"}", &opt_strip());
    assert!(o.ok, "case: {:?}", o.error);
    assert_eq!(
        o.output.unwrap().trim_end(),
        "{\n  \"name\": \"a\\u0001b\"\n}"
    );
}

#[test]
fn strip_removes_control_chars_outside_strings() {
    // 字符串外剔除（\t \n \r 合法空白保留）
    let o = process("{\u{c}\"a\": 1}", &opt_strip());
    assert!(o.ok, "case: {:?}", o.error);
    let out = o.output.unwrap();
    assert!(out.contains("\"a\": 1"));
}

#[test]
fn strip_preserves_legal_whitespace_outside() {
    let o = process("{\n\t\"a\": 1\n}", &opt_strip());
    assert!(o.ok);
}

#[test]
fn strip_escapes_tab_newline_inside_string() {
    let o = process("\"a\tb\nc\"", &opt_strip());
    assert!(o.ok, "case: {:?}", o.error);
    assert_eq!(o.output.unwrap().trim_end(), "\"a\\tb\\nc\"");
}

#[test]
fn strip_reports_warning_and_repaired() {
    let o = process("{\u{1}\"a\": 1}", &opt_strip());
    assert!(o.ok);
    assert!(o.repaired);
    assert!(o
        .warnings
        .iter()
        .any(|w| w.contains("invisible control")));
}

#[test]
fn strip_off_leaves_input_invalid() {
    let o = process("{\u{1}\"a\": 1}", &Default::default());
    assert!(!o.ok);
}

use super::outcome::JsonErrorInfo;

/// 严格解析（不做任何容错）。`serde_json::from_str` 会拒绝尾随非空白字符。
pub fn parse_strict(s: &str) -> Result<serde_json::Value, JsonErrorInfo> {
    serde_json::from_str::<serde_json::Value>(s).map_err(|e| JsonErrorInfo {
        message: e.to_string(),
        line: e.line(),
        column: e.column(),
        category: format!("{:?}", e.classify()),
    })
}

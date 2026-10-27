use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonErrorInfo {
    pub message: String,
    /// 1-based
    pub line: usize,
    /// 1-based
    pub column: usize,
    pub category: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonStats {
    pub bytes_in: usize,
    pub bytes_out: usize,
    pub keys: usize,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonOutcome {
    pub ok: bool,
    /// 输入为空（不算错误，前端显示中性提示）
    pub empty: bool,
    pub output: Option<String>,
    pub error: Option<JsonErrorInfo>,
    /// true = 原始输入不是合法 JSON，经容错预处理后才解析成功
    pub repaired: bool,
    pub warnings: Vec<String>,
    pub stats: Option<JsonStats>,
    /// 解析（及可选排序）后的结构，前端树形渲染用
    pub value: Option<serde_json::Value>,
}

impl JsonOutcome {
    pub fn empty_input() -> Self {
        Self {
            ok: false,
            empty: true,
            output: None,
            error: None,
            repaired: false,
            warnings: Vec::new(),
            stats: None,
            value: None,
        }
    }

    pub fn invalid(err: JsonErrorInfo) -> Self {
        Self {
            ok: false,
            empty: false,
            output: None,
            error: Some(err),
            repaired: false,
            warnings: Vec::new(),
            stats: None,
            value: None,
        }
    }
}

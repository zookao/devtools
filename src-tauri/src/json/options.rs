use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Indent {
    TwoSpaces,
    FourSpaces,
    Tab,
    Minify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct JsonOptions {
    pub indent: Indent,
    /// 递归排序所有对象的 key
    pub sort_keys: bool,
    /// 容错：去掉尾随逗号（[1,2,] / {"a":1,}）
    pub allow_trailing_commas: bool,
    /// 容错：去掉 // 与 /* */ 注释（字符串内除外）
    pub allow_comments: bool,
    /// 容错：给缺失引号的裸键补引号（{a: 1} → {"a": 1}）
    pub auto_repair: bool,
    /// 容错：清理不可见控制字符（U+0000–U+001F）。
    /// 字符串外剔除（保留 \t \n \r 合法空白），字符串内转义为 \uXXXX 保数据。
    pub strip_control_chars: bool,
}

impl Default for JsonOptions {
    fn default() -> Self {
        Self {
            indent: Indent::TwoSpaces,
            sort_keys: false,
            allow_trailing_commas: false,
            allow_comments: false,
            auto_repair: false,
            strip_control_chars: false,
        }
    }
}

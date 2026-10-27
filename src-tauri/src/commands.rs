use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::codec;
use crate::hash;
use crate::ip;
use crate::json::{self, JsonOptions};
use crate::json2md;
use crate::jwt;
use crate::password::{self, PasswordOptions};
use crate::unicode;
use crate::uuid::{self, UuidOptions};
use crate::yaml;

/// JSON 解析/格式化/校验/修复。async：CPU 密集工作不阻塞主线程。
#[tauri::command]
pub async fn json_process(input: String, opts: JsonOptions) -> json::JsonOutcome {
    json::process(&input, &opts)
}

/// JSON（可含 `//` 注释）→ Markdown 参数表格
#[tauri::command]
pub fn json2md_convert(input: String) -> Result<String, String> {
    json2md::convert(&input)
}

/// JSON → YAML
#[tauri::command]
pub fn json_to_yaml(input: String) -> Result<String, String> {
    yaml::json_to_yaml(&input)
}

/// YAML → JSON（2 空格缩进）
#[tauri::command]
pub fn yaml_to_json(input: String) -> Result<String, String> {
    yaml::yaml_to_json(&input)
}

/// 计算文本的 MD5（32 位小写十六进制）
#[tauri::command]
pub fn hash_md5(input: String) -> String {
    hash::md5_hex(&input)
}

/// Base64 编码（UTF-8 文本 → 标准 Base64）
#[tauri::command]
pub fn base64_encode(input: String) -> String {
    codec::encode(&input)
}

/// Base64 解码（标准 Base64 → UTF-8 文本）
#[tauri::command]
pub fn base64_decode(input: String) -> Result<String, String> {
    codec::decode(&input)
}

/// 文本 → \uXXXX 转义序列
#[tauri::command]
pub fn unicode_encode(input: String) -> String {
    unicode::encode(&input)
}

/// \uXXXX 转义序列 → 文本
#[tauri::command]
pub fn unicode_decode(input: String) -> String {
    unicode::decode(&input)
}

/// 批量生成随机密码
#[tauri::command]
pub fn password_generate(opts: PasswordOptions) -> Result<Vec<String>, String> {
    password::generate(&opts)
}

/// 批量生成 UUID v4
#[tauri::command]
pub fn uuid_generate(opts: UuidOptions) -> Vec<String> {
    uuid::generate(&opts)
}

/// 解析 JWT（不校验签名）
#[tauri::command]
pub fn jwt_decode(token: String) -> jwt::JwtDecoded {
    jwt::decode(&token)
}

/// 查询本机公网 IP 与归属地
#[tauri::command]
pub async fn ip_lookup() -> Result<ip::IpReport, String> {
    ip::lookup().await
}

#[tauri::command]
pub fn clipboard_read_text(app: AppHandle) -> Result<String, String> {
    app.clipboard().read_text().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clipboard_write_text(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

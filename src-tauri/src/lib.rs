mod codec;
mod commands;
mod hash;
mod ip;
mod json;
mod json2md;
mod jwt;
mod password;
mod unicode;
mod uuid;
mod yaml;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            commands::json_process,
            commands::json2md_convert,
            commands::json_to_yaml,
            commands::yaml_to_json,
            commands::hash_md5,
            commands::base64_encode,
            commands::base64_decode,
            commands::unicode_encode,
            commands::unicode_decode,
            commands::password_generate,
            commands::uuid_generate,
            commands::jwt_decode,
            commands::ip_lookup,
            commands::clipboard_read_text,
            commands::clipboard_write_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

//! JSON 处理主流程：严格解析 → 失败时按选项容错修复 → 排序/渲染。
//! 错误位置始终按【原始输入】的行列报告（修复产生的偏移不暴露给用户）。

pub mod format;
pub mod options;
pub mod outcome;
pub mod repair;
pub mod validate;
#[cfg(test)]
mod tests;

pub use options::JsonOptions;
pub use outcome::{JsonErrorInfo, JsonOutcome, JsonStats};

pub fn process(input: &str, opts: &JsonOptions) -> JsonOutcome {
    // 去掉 BOM
    let src = input.strip_prefix('\u{feff}').unwrap_or(input);
    if src.trim().is_empty() {
        return JsonOutcome::empty_input();
    }

    match validate::parse_strict(src) {
        Ok(mut value) => finish(src, &mut value, false, Vec::new(), opts),
        Err(strict_err) => {
            let can_repair = opts.allow_comments
                || opts.allow_trailing_commas
                || opts.auto_repair
                || opts.strip_control_chars;
            if !can_repair {
                return JsonOutcome::invalid(strict_err);
            }
            let pre = repair::preprocess(src, opts);
            if !pre.changed {
                return JsonOutcome::invalid(strict_err);
            }
            match validate::parse_strict(&pre.text) {
                Ok(mut value) => finish(src, &mut value, true, pre.notes, opts),
                // 修复后仍失败：仍报原始错误（行列对得上用户看到的输入）
                Err(_) => JsonOutcome::invalid(strict_err),
            }
        }
    }
}

fn finish(
    src: &str,
    value: &mut serde_json::Value,
    repaired: bool,
    mut warnings: Vec<String>,
    opts: &JsonOptions,
) -> JsonOutcome {
    if repaired {
        warnings.insert(0, "Input was not valid JSON; auto-repaired.".to_string());
    }
    if opts.sort_keys {
        format::sort_keys(value);
    }
    match format::render(value, opts.indent) {
        Ok(output) => {
            let stats = JsonStats {
                bytes_in: src.len(),
                bytes_out: output.len(),
                keys: format::count_keys(value),
                depth: format::depth(value),
            };
            JsonOutcome {
                ok: true,
                empty: false,
                output: Some(output),
                error: None,
                repaired,
                warnings,
                stats: Some(stats),
                value: Some(value.clone()),
            }
        }
        Err(e) => JsonOutcome::invalid(JsonErrorInfo {
            message: e.to_string(),
            line: e.line(),
            column: e.column(),
            category: "io".to_string(),
        }),
    }
}

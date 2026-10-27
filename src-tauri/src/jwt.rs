//! JWT 解析：Base64URL 解码头部与载荷，提取标准时间声明。
//!
//! 只做解码，不校验签名（客户端没有密钥，也不该假装验证）。

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtDecoded {
    pub valid: bool,
    pub error: Option<String>,
    /// 格式化后的 JSON 文本
    pub header: Option<String>,
    pub payload: Option<String>,
    pub signature: Option<String>,
    pub issued_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub not_before: Option<i64>,
}

fn fail(msg: impl Into<String>) -> JwtDecoded {
    JwtDecoded {
        valid: false,
        error: Some(msg.into()),
        ..Default::default()
    }
}

fn b64url_decode(s: &str, label: &str) -> Result<Vec<u8>, String> {
    URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|_| format!("{label}不是合法的 Base64URL 编码"))
}

/// 把 `Value` 里的数字声明读成 i64（兼容浮点写法）
fn num_claim(v: &Value, key: &str) -> Option<i64> {
    let n = v.get(key)?;
    n.as_i64().or_else(|| n.as_f64().map(|f| f as i64))
}

pub fn decode(token: &str) -> JwtDecoded {
    let t = token.trim();
    if t.is_empty() {
        return fail("令牌为空");
    }

    let parts: Vec<&str> = t.split('.').collect();
    if parts.len() != 3 {
        return fail(format!("JWT 应包含 3 段，当前为 {} 段", parts.len()));
    }

    let header_raw = match b64url_decode(parts[0], "头部") {
        Ok(b) => b,
        Err(e) => return fail(e),
    };
    let header: Value = match serde_json::from_slice(&header_raw) {
        Ok(v) => v,
        Err(_) => return fail("头部不是合法的 JSON"),
    };

    let payload_raw = match b64url_decode(parts[1], "载荷") {
        Ok(b) => b,
        Err(e) => return fail(e),
    };
    let payload: Value = match serde_json::from_slice(&payload_raw) {
        Ok(v) => v,
        Err(_) => return fail("载荷不是合法的 JSON"),
    };

    if parts[2].is_empty() {
        return fail("缺少签名段");
    }

    JwtDecoded {
        valid: true,
        error: None,
        header: serde_json::to_string_pretty(&header).ok(),
        payload: serde_json::to_string_pretty(&payload).ok(),
        signature: Some(parts[2].to_string()),
        issued_at: num_claim(&payload, "iat"),
        expires_at: num_claim(&payload, "exp"),
        not_before: num_claim(&payload, "nbf"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn b64(v: &Value) -> String {
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(v).unwrap())
    }

    fn token(header: &Value, payload: &Value) -> String {
        format!("{}.{}.{}", b64(header), b64(payload), "sig_placeholder")
    }

    #[test]
    fn decodes_standard_token() {
        let t = token(
            &json!({"alg": "HS256", "typ": "JWT"}),
            &json!({"sub": "1234567890", "iat": 1793003227, "exp": 1793009827}),
        );
        let d = decode(&t);
        assert!(d.valid, "{:?}", d.error);
        assert!(d.header.unwrap().contains("\"HS256\""));
        assert_eq!(d.issued_at, Some(1793003227));
        assert_eq!(d.expires_at, Some(1793009827));
        assert_eq!(d.signature.as_deref(), Some("sig_placeholder"));
    }

    #[test]
    fn preserves_key_order() {
        let t = token(
            &json!({"alg": "HS256"}),
            &json!({"sub": "1", "name": "dev", "iat": 1}),
        );
        let p = decode(&t).payload.unwrap();
        let sub = p.find("\"sub\"").unwrap();
        let name = p.find("\"name\"").unwrap();
        assert!(sub < name);
    }

    #[test]
    fn wrong_segment_count() {
        let d = decode("abc.def");
        assert!(!d.valid);
        assert!(d.error.unwrap().contains("3 段"));
    }

    #[test]
    fn empty_token() {
        assert!(!decode("   ").valid);
    }

    #[test]
    fn invalid_base64_header() {
        let d = decode("!!!.e30.sig");
        assert!(!d.valid);
        assert!(d.error.unwrap().contains("头部"));
    }

    #[test]
    fn header_not_json() {
        let h = URL_SAFE_NO_PAD.encode(b"not json");
        let p = b64(&json!({"a": 1}));
        let d = decode(&format!("{h}.{p}.sig"));
        assert!(!d.valid);
        assert!(d.error.unwrap().contains("头部"));
    }

    #[test]
    fn missing_signature() {
        let t = format!(
            "{}.{}.",
            b64(&json!({"alg": "none"})),
            b64(&json!({"sub": "1"}))
        );
        let d = decode(&t);
        assert!(!d.valid);
        assert!(d.error.unwrap().contains("签名"));
    }

    #[test]
    fn float_exp_is_accepted() {
        let t = token(
            &json!({"alg": "HS256"}),
            &json!({"exp": 1793009827.5}),
        );
        assert_eq!(decode(&t).expires_at, Some(1793009827));
    }

    #[test]
    fn no_time_claims() {
        let t = token(&json!({"alg": "HS256"}), &json!({"sub": "1"}));
        let d = decode(&t);
        assert!(d.valid);
        assert_eq!(d.issued_at, None);
        assert_eq!(d.expires_at, None);
    }
}

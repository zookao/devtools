//! 本机公网 IP 查询：分别探测 IPv4 / IPv6 出口地址，再查询归属地。
//!
//! 不依赖本地网卡信息（那只能拿到内网地址），而是请求外部服务读取出口 IP。

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

/// 统一的查询结果，字段都用 Option：不同服务能提供的信息不完全一致
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpInfo {
    pub ip: String,
    /// IPv4 / IPv6
    pub ip_type: Option<String>,
    pub continent: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub postal: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub utc_offset: Option<String>,
    pub isp: Option<String>,
    pub org: Option<String>,
    pub asn: Option<String>,
    /// 数据来源服务域名
    pub source: String,
}

/// 同时给出 IPv4 / IPv6 出口地址，前端可切换查看
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpReport {
    pub ipv4: Option<IpInfo>,
    pub ipv6: Option<IpInfo>,
    /// 单个地址族失败时的提示（如当前网络不支持 IPv6），不阻塞整体结果
    pub errors: Vec<String>,
}

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) devtools/0.1";

/// 纯文本回显出口地址的服务，按优先级排列
const IPV4_ENDPOINTS: [&str; 2] = ["https://ipv4.icanhazip.com", "https://checkip.amazonaws.com"];
const IPV6_ENDPOINTS: [&str; 1] = ["https://ipv6.icanhazip.com"];

pub async fn lookup() -> Result<IpReport, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("创建网络客户端失败: {e}"))?;

    let mut errors: Vec<String> = Vec::new();
    let ipv4 = fetch_family(&client, "IPv4", &IPV4_ENDPOINTS, &mut errors).await;
    let ipv6 = fetch_family(&client, "IPv6", &IPV6_ENDPOINTS, &mut errors).await;

    if ipv4.is_none() && ipv6.is_none() {
        return Err(format!("未能获取到公网 IP\n{}", errors.join("\n")));
    }

    Ok(IpReport { ipv4, ipv6, errors })
}

/// 探测某个地址族的出口 IP 并查询归属地；归属地查询失败时仍返回 IP 本身
async fn fetch_family(
    client: &reqwest::Client,
    label: &str,
    endpoints: &[&str],
    errors: &mut Vec<String>,
) -> Option<IpInfo> {
    let ip = match discover(client, label, endpoints).await {
        Ok(ip) => ip,
        Err(e) => {
            errors.push(format!("{label}: {e}"));
            return None;
        }
    };

    match geolocate(client, &ip).await {
        Ok(mut info) => {
            info.ip_type = Some(label.to_string());
            Some(info)
        }
        Err(e) => {
            errors.push(format!("{label} 归属地: {e}"));
            Some(IpInfo {
                ip,
                ip_type: Some(label.to_string()),
                ..Default::default()
            })
        }
    }
}

async fn discover(
    client: &reqwest::Client,
    label: &str,
    endpoints: &[&str],
) -> Result<String, String> {
    let mut errors: Vec<String> = Vec::new();
    for url in endpoints {
        match client.get(*url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(body) => {
                    let ip = body.trim().to_string();
                    let valid = if label == "IPv6" {
                        ip.contains(':') && ip.parse::<std::net::Ipv6Addr>().is_ok()
                    } else {
                        ip.parse::<std::net::Ipv4Addr>().is_ok()
                    };
                    if valid {
                        return Ok(ip);
                    }
                    errors.push(format!("{url}: 返回内容不是 {label} 地址"));
                }
                Err(e) => errors.push(format!("{url}: 读取响应失败 ({e})")),
            },
            Ok(resp) => errors.push(format!("{url}: HTTP {}", resp.status().as_u16())),
            Err(e) => errors.push(format!("{url}: 请求失败 ({e})")),
        }
    }
    Err(format!("无法获取地址\n{}", errors.join("\n")))
}

/// 按 IP 查询归属地，ipwho.is 优先，ipinfo.io 兜底
async fn geolocate(client: &reqwest::Client, ip: &str) -> Result<IpInfo, String> {
    let sources: [(&str, String, fn(&str) -> Result<IpInfo, String>); 2] = [
        ("ipwho.is", format!("https://ipwho.is/{ip}"), parse_ipwho),
        ("ipinfo.io", format!("https://ipinfo.io/{ip}/json"), parse_ipinfo),
    ];

    let mut errors: Vec<String> = Vec::new();
    for (host, url, parse) in sources {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(body) => match parse(&body) {
                    Ok(mut info) => {
                        info.source = host.to_string();
                        return Ok(info);
                    }
                    Err(e) => errors.push(format!("{host}: {e}")),
                },
                Err(e) => errors.push(format!("{host}: 读取响应失败 ({e})")),
            },
            Ok(resp) => errors.push(format!("{host}: HTTP {}", resp.status().as_u16())),
            Err(e) => errors.push(format!("{host}: 请求失败 ({e})")),
        }
    }
    Err(errors.join("\n"))
}


/// 空字符串视为缺失
fn non_empty(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|v| !v.is_empty() && *v != "-")
        .map(str::to_string)
}

fn detect_ip_type(ip: &str) -> Option<String> {
    if ip.contains(':') {
        Some("IPv6".to_string())
    } else if ip.contains('.') {
        Some("IPv4".to_string())
    } else {
        None
    }
}

fn json_str(v: &Value, path: &[&str]) -> Option<String> {
    let mut cur = v;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_str().and_then(|s| non_empty(Some(s)))
}

fn json_num(v: &Value, path: &[&str]) -> Option<f64> {
    let mut cur = v;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_f64()
}

fn json_raw_str(v: &Value, path: &[&str]) -> Option<String> {
    let mut cur = v;
    for key in path {
        cur = cur.get(*key)?;
    }
    match cur {
        Value::String(s) => non_empty(Some(s)),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// ipwho.is：字段最全，优先使用
fn parse_ipwho(body: &str) -> Result<IpInfo, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| format!("响应不是合法 JSON ({e})"))?;
    if v.get("success").and_then(Value::as_bool) == Some(false) {
        return Err("服务返回失败".to_string());
    }
    let ip = json_str(&v, &["ip"]).ok_or_else(|| "响应缺少 ip 字段".to_string())?;

    Ok(IpInfo {
        ip_type: json_str(&v, &["type"]).or_else(|| detect_ip_type(&ip)),
        continent: json_str(&v, &["continent"]),
        country: json_str(&v, &["country"]),
        country_code: json_str(&v, &["country_code"]),
        region: json_str(&v, &["region"]),
        city: json_str(&v, &["city"]),
        postal: json_str(&v, &["postal"]),
        latitude: json_num(&v, &["latitude"]),
        longitude: json_num(&v, &["longitude"]),
        timezone: json_str(&v, &["timezone", "id"]),
        utc_offset: json_str(&v, &["timezone", "utc"]),
        isp: json_str(&v, &["connection", "isp"]),
        org: json_str(&v, &["connection", "org"]),
        asn: json_raw_str(&v, &["connection", "asn"]),
        source: String::new(),
        ip,
    })
}

/// ipinfo.io：loc 为 "纬度,经度"，org 形如 "AS4808 China Unicom"
fn parse_ipinfo(body: &str) -> Result<IpInfo, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| format!("响应不是合法 JSON ({e})"))?;
    let ip = json_str(&v, &["ip"]).ok_or_else(|| "响应缺少 ip 字段".to_string())?;

    let (latitude, longitude) = json_str(&v, &["loc"])
        .and_then(|loc| {
            let (lat, lon) = loc.split_once(',')?;
            Some((lat.trim().parse::<f64>().ok()?, lon.trim().parse::<f64>().ok()?))
        })
        .map(|(lat, lon)| (Some(lat), Some(lon)))
        .unwrap_or((None, None));

    // org 首段是 ASN，其余是运营商名称
    let org = json_str(&v, &["org"]);
    let asn = org.as_ref().and_then(|o| {
        let first = o.split_whitespace().next()?;
        first.starts_with("AS").then(|| first[2..].to_string())
    });
    let isp = org.as_ref().and_then(|o| {
        let rest = o.split_once(' ').map(|(_, r)| r.trim()).unwrap_or("");
        non_empty(Some(rest))
    });

    Ok(IpInfo {
        ip_type: detect_ip_type(&ip),
        country: json_str(&v, &["country"]),
        country_code: json_str(&v, &["country"]),
        region: json_str(&v, &["region"]),
        city: json_str(&v, &["city"]),
        postal: json_str(&v, &["postal"]),
        latitude,
        longitude,
        timezone: json_str(&v, &["timezone"]),
        isp,
        org,
        asn,
        source: String::new(),
        ip,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const IPWHO: &str = r#"{
        "ip": "123.119.177.64",
        "success": true,
        "type": "IPv4",
        "continent": "Asia",
        "country": "China",
        "country_code": "CN",
        "region": "Beijing",
        "city": "Beijing",
        "latitude": 39.9074752,
        "longitude": 116.3972298,
        "postal": "102600",
        "connection": {"asn": 4808, "org": "China Unicom Beijing province network",
                       "isp": "China Unicom Beijing Province Network"},
        "timezone": {"id": "Asia/Shanghai", "utc": "+08:00"}
    }"#;

    #[test]
    fn parses_ipwho_response() {
        let info = parse_ipwho(IPWHO).unwrap();
        assert_eq!(info.ip, "123.119.177.64");
        assert_eq!(info.ip_type.as_deref(), Some("IPv4"));
        assert_eq!(info.country.as_deref(), Some("China"));
        assert_eq!(info.city.as_deref(), Some("Beijing"));
        assert_eq!(info.timezone.as_deref(), Some("Asia/Shanghai"));
        assert_eq!(info.utc_offset.as_deref(), Some("+08:00"));
        assert_eq!(info.asn.as_deref(), Some("4808"));
        assert_eq!(info.isp.as_deref(), Some("China Unicom Beijing Province Network"));
        assert!((info.latitude.unwrap() - 39.9074752).abs() < 1e-9);
    }

    #[test]
    fn ipwho_failure_is_error() {
        assert!(parse_ipwho(r#"{"ip":"1.1.1.1","success":false}"#).is_err());
        assert!(parse_ipwho(r#"{"success":true}"#).is_err());
    }

    #[test]
    fn parses_ipinfo_response() {
        let body = r#"{
            "ip": "8.8.8.8", "city": "Mountain View", "region": "California",
            "country": "US", "loc": "37.4056,-122.0775", "postal": "94043",
            "timezone": "America/Los_Angeles",
            "org": "AS15169 Google LLC"
        }"#;
        let info = parse_ipinfo(body).unwrap();
        assert_eq!(info.ip, "8.8.8.8");
        assert_eq!(info.asn.as_deref(), Some("15169"));
        assert_eq!(info.isp.as_deref(), Some("Google LLC"));
        assert_eq!(info.latitude, Some(37.4056));
        assert_eq!(info.longitude, Some(-122.0775));
        assert_eq!(info.country_code.as_deref(), Some("US"));
    }

    #[test]
    fn parses_ipinfo_without_optional_fields() {
        let info = parse_ipinfo(r#"{"ip":"1.1.1.1"}"#).unwrap();
        assert_eq!(info.ip, "1.1.1.1");
        assert_eq!(info.latitude, None);
        assert_eq!(info.asn, None);
    }

    #[test]
    fn detects_ip_version() {
        assert_eq!(detect_ip_type("1.1.1.1").as_deref(), Some("IPv4"));
        assert_eq!(detect_ip_type("2400:3200::1").as_deref(), Some("IPv6"));
        assert_eq!(detect_ip_type("unknown"), None);
    }

    #[test]
    fn validates_discovered_ip_format() {
        assert!("111.199.191.18".parse::<std::net::Ipv4Addr>().is_ok());
        assert!("2408:8207:7821:d0ec:2c6b:36a0:1ba9:455".parse::<std::net::Ipv6Addr>().is_ok());
        assert!("2408:8207::1".contains(':'));
        assert!("<html>".parse::<std::net::Ipv4Addr>().is_err());
    }
}

use axum::extract::ConnectInfo;
use axum::{body::Body, extract::Request};
use once_cell::sync::Lazy;
use regex::Regex;
use std::net::IpAddr;
use std::net::SocketAddr;
use woothee::parser::Parser;

#[cfg(feature = "cloudflare")]
use axum::http::HeaderMap;

static USER_AGENT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?iU)^(?:(?:wakatime|chrome|firefox|edge)\/(?:v?[\d+.]+|unset)?\s)(?:\(?(\w+)[-_].*\)?.+\s)?(?:([^\/\s]+)\/[\w\d\.]+\s)?([^\/\s]+)-wakatime\/.+$").unwrap()
});

/// Parse user agent into OS and editor
/// Based on https://github.com/muety/wakapi/blob/master/utils/http.go#L89-L127
pub fn parse_user_agent(ua: String) -> Result<(String, String), String> {
    // try to parse wakatime client user agents first
    if let Some(groups) = USER_AGENT_PATTERN.captures(&ua)
        && groups.len() == 4
    {
        // extract OS
        let os = groups
            .get(1)
            .map_or("", |m| m.as_str())
            .to_ascii_lowercase();

        // parse editor
        let mut editor = groups
            .get(2)
            .map_or("", |m| m.as_str())
            .to_ascii_lowercase();
        if editor.is_empty() {
            editor = groups
                .get(3)
                .map_or("", |m| m.as_str())
                .to_ascii_lowercase();
        }

        return Ok((os, editor));
    }

    // try to parse the browser user agent as a fallback using woothee
    let parser = Parser::new();
    if let Some(result) = parser.parse(&ua)
        && !result.name.is_empty()
    {
        let os = if !result.os.is_empty() {
            if ua.to_ascii_lowercase().contains("windows") {
                "windows".to_string()
            } else {
                result.os.to_ascii_lowercase()
            }
        } else {
            return Err("failed to parse user agent string".to_string());
        };

        return Ok((os, result.name.to_ascii_lowercase()));
    }

    Err("failed to parse user agent string".to_string())
}

/// Extract client IP from request headers or connection info
pub fn extract_client_ip(request: &Request<Body>) -> IpAddr {
    #[cfg(feature = "cloudflare")]
    return extract_client_ip_cloudflare(request.headers())
        .unwrap_or_else(|| extract_direct_client_ip(request));

    #[cfg(not(feature = "cloudflare"))]
    extract_direct_client_ip(request)
}

/// Extract direct client IP from connection info
fn extract_direct_client_ip(request: &Request<Body>) -> IpAddr {
    if let Some(ConnectInfo(addr)) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        addr.ip()
    } else {
        IpAddr::from([0, 0, 0, 0])
    }
}

/// Extract client ip from cloudflare header
#[cfg(feature = "cloudflare")]
pub fn extract_client_ip_cloudflare(headers: &HeaderMap) -> Option<IpAddr> {
    if let Some(cf_ip) = headers.get("cf-connecting-ip") {
        if let Ok(ip_str) = cf_ip.to_str() {
            return ip_str.parse().ok();
        }
    }
    None
}

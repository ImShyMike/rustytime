use axum::extract::ConnectInfo;
use axum::http::HeaderMap;
use axum::{body::Body, extract::Request};
use once_cell::sync::Lazy;
use regex::Regex;
use std::net::IpAddr;
use std::net::SocketAddr;
use tower_governor::{
    errors::GovernorError,
    key_extractor::{KeyExtractor, PeerIpKeyExtractor, SmartIpKeyExtractor},
};
use woothee::parser::Parser;

use crate::utils::env::use_cloudflare_headers;

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
#[inline(always)]
pub fn extract_client_ip(request: &Request<Body>) -> IpAddr {
    if use_cloudflare_headers()
        && let Some(ip) = extract_client_ip_cloudflare(request.headers())
    {
        return ip;
    }

    extract_direct_client_ip(request)
}

/// Resolve client IP from request headers or connection info
#[inline(always)]
pub fn extract_client_ip_from_headers(headers: &HeaderMap, addr: SocketAddr) -> IpAddr {
    if use_cloudflare_headers()
        && let Some(ip) = extract_client_ip_cloudflare(headers)
    {
        return ip;
    }

    addr.ip()
}

/// Extract direct client IP from connection info
#[inline(always)]
fn extract_direct_client_ip(request: &Request<Body>) -> IpAddr {
    if let Some(ConnectInfo(addr)) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        addr.ip()
    } else {
        IpAddr::from([0, 0, 0, 0])
    }
}

/// Extract client ip from cloudflare header
#[inline(always)]
pub fn extract_client_ip_cloudflare(headers: &HeaderMap) -> Option<IpAddr> {
    if let Some(cf_ip) = headers.get("cf-connecting-ip")
        && let Ok(ip_str) = cf_ip.to_str()
    {
        return ip_str.parse().ok();
    }
    None
}

#[derive(Clone, Copy)]
pub struct CloudflareAwareKeyExtractor {
    use_cloudflare: bool,
}

impl CloudflareAwareKeyExtractor {
    #[inline(always)]
    pub const fn new(use_cloudflare: bool) -> Self {
        Self { use_cloudflare }
    }
}

impl KeyExtractor for CloudflareAwareKeyExtractor {
    type Key = IpAddr;

    fn name(&self) -> &'static str {
        if self.use_cloudflare {
            "cloudflare-smart-ip"
        } else {
            "peer-ip"
        }
    }

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        if self.use_cloudflare {
            SmartIpKeyExtractor.extract(req)
        } else {
            PeerIpKeyExtractor.extract(req)
        }
    }

    fn key_name(&self, key: &Self::Key) -> Option<String> {
        Some(key.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wakatime_user_agent_signature_linux() {
        let user_agent =
            "wakatime/v1.115.2 (linux-6.14.1) go1.24.2 vscode/1.100.0 vscode-wakatime/25.0.3"
                .to_string();
        let (os, editor) = parse_user_agent(user_agent).unwrap();
        assert_eq!(os, "linux".to_string());
        assert_eq!(editor, "vscode".to_string());
    }

    #[test]
    fn parses_wakatime_user_agent_signature_windows() {
        let user_agent = "wakatime/v1.115.2 (windows-10.0.26100.1742-x86_64) go1.24.2 vscode/1.100.2 vscode-wakatime/25.0.3".to_string();
        let (os, editor) = parse_user_agent(user_agent).unwrap();
        assert_eq!(os, "windows".to_string());
        assert_eq!(editor, "vscode".to_string());
    }

    #[test]
    fn parses_wakatime_user_agent_signature_darwin() {
        let user_agent =
            "wakatime/v1.131.0 (darwin-24.6.0-arm64) go1.24.4 zsh/5.9 terminal-wakatime/v1.1.5"
                .to_string();
        let (os, editor) = parse_user_agent(user_agent).unwrap();
        assert_eq!(os, "darwin".to_string());
        assert_eq!(editor, "zsh".to_string());
    }

    #[test]
    fn falls_back_to_woothee_for_browser_user_agents() {
        let browser_ua =
            "Mozilla/5.0 (X11; Linux x86_64; rv:144.0) Gecko/20100101 Firefox/144.0".to_string();
        let (os, editor) = parse_user_agent(browser_ua).unwrap();
        assert_eq!(os, "linux".to_string());
        assert_eq!(editor, "firefox".to_string());
    }

    #[test]
    fn extracts_client_ip_from_cloudflare_headers_when_enabled() {
        let headers = {
            let mut map = HeaderMap::new();
            map.insert("cf-connecting-ip", "100.1.1.21".parse().unwrap());
            map
        };
        let ip = extract_client_ip_cloudflare(&headers).unwrap();
        assert_eq!(ip, "100.1.1.21".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn extracts_client_ip_from_connection_info_when_disabled() {
        let mut req = Request::builder()
            .uri("http://example.com")
            .body(Body::empty())
            .unwrap();

        let sock: SocketAddr = "1.1.1.1:4242".parse().unwrap();
        req.extensions_mut().insert(ConnectInfo(sock));

        let ip = extract_direct_client_ip(&req);
        assert_eq!(ip, "1.1.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn cloudflare_key_extractor_switches_between_modes() {
        let extractor_cf = CloudflareAwareKeyExtractor::new(true);
        assert_eq!(extractor_cf.name(), "cloudflare-smart-ip");
    }
}

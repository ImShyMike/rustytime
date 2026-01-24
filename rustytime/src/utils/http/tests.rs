use super::*;

// ==================== parse_user_agent tests ====================

#[test]
fn parses_wakatime_user_agent_signature_linux() {
    let user_agent =
        "wakatime/v1.115.2 (linux-6.14.1) go1.24.2 vscode/1.100.0 vscode-wakatime/25.0.3"
            .to_string();
    let (os, editor) = parse_user_agent(user_agent).unwrap();
    assert_eq!(os, Some("linux".to_string()));
    assert_eq!(editor, Some("vscode".to_string()));
}

#[test]
fn parses_wakatime_user_agent_signature_windows() {
    let user_agent = "wakatime/v1.115.2 (windows-10.0.26100.1742-x86_64) go1.24.2 vscode/1.100.2 vscode-wakatime/25.0.3".to_string();
    let (os, editor) = parse_user_agent(user_agent).unwrap();
    assert_eq!(os, Some("windows".to_string()));
    assert_eq!(editor, Some("vscode".to_string()));
}

#[test]
fn parses_wakatime_user_agent_signature_darwin() {
    let user_agent =
        "wakatime/v1.131.0 (darwin-24.6.0-arm64) go1.24.4 zsh/5.9 terminal-wakatime/v1.1.5"
            .to_string();
    let (os, editor) = parse_user_agent(user_agent).unwrap();
    assert_eq!(os, Some("darwin".to_string()));
    assert_eq!(editor, Some("zsh".to_string()));
}

#[test]
fn parses_neovim_wakatime_user_agent() {
    let user_agent =
        "wakatime/v1.102.1 (linux-6.8.0-45-generic-x86_64) go1.23.2 neovim/0.10.0 vim-wakatime/11.1.3"
            .to_string();
    let (os, editor) = parse_user_agent(user_agent).unwrap();
    assert_eq!(os, Some("linux".to_string()));
    assert_eq!(editor, Some("neovim".to_string()));
}

#[test]
fn parses_jetbrains_intellij_wakatime_user_agent() {
    let user_agent =
        "wakatime/v1.98.0 (darwin-23.5.0-arm64) go1.22.5 intellij/2024.1 intellij-wakatime/15.0.1"
            .to_string();
    let (os, editor) = parse_user_agent(user_agent).unwrap();
    assert_eq!(os, Some("darwin".to_string()));
    assert_eq!(editor, Some("intellij".to_string()));
}

#[test]
fn parses_jetbrains_pycharm_wakatime_user_agent() {
    let user_agent =
        "wakatime/v1.98.0 (windows-10.0.19045-x86_64) go1.22.5 pycharm/2024.2 pycharm-wakatime/15.0.1"
            .to_string();
    let (os, editor) = parse_user_agent(user_agent).unwrap();
    assert_eq!(os, Some("windows".to_string()));
    assert_eq!(editor, Some("pycharm".to_string()));
}

#[test]
fn parses_sublime_text_wakatime_user_agent() {
    let user_agent =
        "wakatime/v1.105.0 (linux-5.15.0-x86_64) go1.23.0 sublime/4180 sublime-wakatime/11.1.0"
            .to_string();
    let (os, editor) = parse_user_agent(user_agent).unwrap();
    assert_eq!(os, Some("linux".to_string()));
    assert_eq!(editor, Some("sublime".to_string()));
}

#[test]
fn parses_vim_wakatime_user_agent() {
    let user_agent =
        "wakatime/v1.100.0 (darwin-22.6.0-x86_64) go1.21.0 vim/9.0 vim-wakatime/11.0.0".to_string();
    let (os, editor) = parse_user_agent(user_agent).unwrap();
    assert_eq!(os, Some("darwin".to_string()));
    assert_eq!(editor, Some("vim".to_string()));
}

#[test]
fn falls_back_to_woothee_for_browser_user_agents() {
    let browser_ua =
        "Mozilla/5.0 (X11; Linux x86_64; rv:144.0) Gecko/20100101 Firefox/144.0".to_string();
    let (os, editor) = parse_user_agent(browser_ua).unwrap();
    assert_eq!(os, Some("linux".to_string()));
    assert_eq!(editor, Some("firefox".to_string()));
}

#[test]
fn parses_chrome_on_windows_user_agent() {
    let browser_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string();
    let (os, editor) = parse_user_agent(browser_ua).unwrap();
    assert_eq!(os, Some("windows".to_string()));
    assert_eq!(editor, Some("chrome".to_string()));
}

#[test]
fn parses_safari_on_macos_user_agent() {
    let browser_ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15".to_string();
    let (os, editor) = parse_user_agent(browser_ua).unwrap();
    assert_eq!(os, Some("mac osx".to_string()));
    assert_eq!(editor, Some("safari".to_string()));
}

#[test]
fn returns_error_for_whitespace_only_user_agent() {
    let result = parse_user_agent("   ".to_string());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "failed to parse user agent string");
}

#[test]
fn returns_error_for_invalid_user_agent() {
    let result = parse_user_agent("completely-invalid-garbage-string".to_string());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "failed to parse user agent string");
}

#[test]
fn handles_very_long_user_agent_string() {
    let long_ua = format!(
        "wakatime/v1.115.2 (linux-6.14.1) go1.24.2 vscode/1.100.0 vscode-wakatime/25.0.3 {}",
        "x".repeat(10000)
    );
    let (os, editor) = parse_user_agent(long_ua).unwrap();
    assert_eq!(os, Some("linux".to_string()));
    assert_eq!(editor, Some("vscode".to_string()));
}

// ==================== extract_client_ip_cloudflare tests ====================

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
fn extracts_ipv6_from_cloudflare_headers() {
    let headers = {
        let mut map = HeaderMap::new();
        map.insert(
            "cf-connecting-ip",
            "2001:db8:85a3::8a2e:370:7334".parse().unwrap(),
        );
        map
    };
    let ip = extract_client_ip_cloudflare(&headers).unwrap();
    assert_eq!(
        ip,
        "2001:db8:85a3::8a2e:370:7334".parse::<IpAddr>().unwrap()
    );
}

#[test]
fn returns_none_for_invalid_ip_in_cloudflare_header() {
    let headers = {
        let mut map = HeaderMap::new();
        map.insert("cf-connecting-ip", "not-an-ip-address".parse().unwrap());
        map
    };
    let result = extract_client_ip_cloudflare(&headers);
    assert!(result.is_none());
}

#[test]
fn returns_none_for_empty_cloudflare_header() {
    let headers = {
        let mut map = HeaderMap::new();
        map.insert("cf-connecting-ip", "".parse().unwrap());
        map
    };
    let result = extract_client_ip_cloudflare(&headers);
    assert!(result.is_none());
}

#[test]
fn returns_none_for_missing_cloudflare_header() {
    let headers = HeaderMap::new();
    let result = extract_client_ip_cloudflare(&headers);
    assert!(result.is_none());
}

// ==================== extract_direct_client_ip tests ====================

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
fn returns_zero_ip_when_connect_info_missing() {
    let req = Request::builder()
        .uri("http://example.com")
        .body(Body::empty())
        .unwrap();

    let ip = extract_direct_client_ip(&req);
    assert_eq!(ip, "0.0.0.0".parse::<IpAddr>().unwrap());
}

#[test]
fn extracts_ipv6_from_connection_info() {
    let mut req = Request::builder()
        .uri("http://example.com")
        .body(Body::empty())
        .unwrap();

    let sock: SocketAddr = "[2001:db8::1]:8080".parse().unwrap();
    req.extensions_mut().insert(ConnectInfo(sock));

    let ip = extract_direct_client_ip(&req);
    assert_eq!(ip, "2001:db8::1".parse::<IpAddr>().unwrap());
}

// ==================== extract_client_ip_from_headers tests ====================

#[test]
fn falls_back_to_socket_addr_when_cf_header_missing() {
    let headers = HeaderMap::new();
    let addr: SocketAddr = "192.168.1.100:12345".parse().unwrap();

    let ip = extract_client_ip_from_headers(&headers, addr);
    assert_eq!(ip, "192.168.1.100".parse::<IpAddr>().unwrap());
}

// ==================== CloudflareAwareKeyExtractor tests ====================

#[test]
fn cloudflare_key_extractor_switches_between_modes() {
    let extractor_cf = CloudflareAwareKeyExtractor::new(true);
    assert_eq!(extractor_cf.name(), "cloudflare-smart-ip");
}

#[test]
fn cloudflare_key_extractor_returns_peer_ip_when_disabled() {
    let extractor = CloudflareAwareKeyExtractor::new(false);
    assert_eq!(extractor.name(), "peer-ip");
}

#[test]
fn cloudflare_key_extractor_key_name_returns_ip_string() {
    let extractor = CloudflareAwareKeyExtractor::new(false);
    let ip: IpAddr = "10.0.0.1".parse().unwrap();
    assert_eq!(extractor.key_name(&ip), Some("10.0.0.1".to_string()));
}

#[test]
fn cloudflare_key_extractor_key_name_returns_ipv6_string() {
    let extractor = CloudflareAwareKeyExtractor::new(true);
    let ip: IpAddr = "::1".parse().unwrap();
    assert_eq!(extractor.key_name(&ip), Some("::1".to_string()));
}

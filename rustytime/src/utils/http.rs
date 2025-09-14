use once_cell::sync::Lazy;
use regex::Regex;
use woothee::parser::Parser;

static USER_AGENT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?iU)^(?:(?:wakatime|chrome|firefox|edge)\/(?:v?[\d+.]+|unset)?\s)(?:\(?(\w+)[-_].*\)?.+\s)?(?:([^\/\s]+)\/[\w\d\.]+\s)?([^\/\s]+)-wakatime\/.+$").unwrap()
});

/// Parse user agent into OS and editor
/// Based on https://github.com/muety/wakapi/blob/master/utils/http.go#L89-L127
pub fn parse_user_agent(ua: String) -> Result<(String, String), String> {
    // try to parse wakatime client user agents first
    if let Some(groups) = USER_AGENT_PATTERN.captures(&ua) {
        if groups.len() == 4 {
            // extract OS
            let os = groups.get(1).map_or("", |m| m.as_str()).to_string();

            // parse editor
            let mut editor = groups.get(2).map_or("", |m| m.as_str()).to_string();
            if editor.is_empty() {
                editor = groups.get(3).map_or("", |m| m.as_str()).to_string();
            }

            return Ok((categorize_os(&os), categorize_editor(&editor)));
        }
    }

    // try to parse the browser user agent as a fallback using woothee
    let parser = Parser::new();
    if let Some(result) = parser.parse(&ua) {
        if !result.name.is_empty() {
            let os = if !result.os.is_empty() {
                result.os.to_string()
            } else if ua.to_lowercase().contains("windows") {
                "Windows".to_string()
            } else {
                return Err("failed to parse user agent string".to_string());
            };

            return Ok((categorize_os(&os), categorize_editor(result.name)));
        }
    }

    Err("failed to parse user agent string".to_string())
}

fn capitalize(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let mut chars: Vec<char> = s.chars().collect();
    chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
    chars.into_iter().collect()
}

fn categorize_os(os: &str) -> String {
    match os.to_lowercase().as_str() {
        "win" => "Windows".to_string(),
        "darwin" => "macOS".to_string(),
        _ => capitalize(os),
    }
}

fn categorize_editor(editor: &str) -> String {
    match editor.to_lowercase().as_str() {
        "vscode" => "VSCode".to_string(),
        "ktexteditor" => "Kate".to_string(),
        _ => capitalize(editor),
    }
}

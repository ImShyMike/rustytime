use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");

    println!("cargo:rustc-env=GIT_SHA={}", git_sha());

    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap_or("unknown".to_string())
    );

    println!(
        "cargo:rustc-env=BUILD_TIMESTAMP={}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    )
}

fn git_sha() -> String {
    let output = match Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return "unknown".to_string(),
    };

    let mut sha = String::from_utf8(output.stdout).unwrap_or_else(|_| "unknown".to_string());
    sha = sha.trim().to_string();

    let status = Command::new("git")
        .args(["diff-index", "--quiet", "HEAD"])
        .status()
        .ok();

    if let Some(status) = status
        && !status.success()
    {
        sha.push_str("-dirty");
    }

    sha
}

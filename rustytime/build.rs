use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");

    if let Some(sha) = git_sha() {
        println!("cargo:rustc-env=GIT_SHA={sha}");
    }

    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );

    println!(
        "cargo:rustc-env=BUILD_TIMESTAMP={}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
}

fn git_sha() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let mut sha = String::from_utf8(output.stdout).ok()?;
    sha = sha.trim().to_string();

    let status = Command::new("git")
        .args(["diff-index", "--quiet", "HEAD"])
        .status()
        .ok()?;

    if !status.success() {
        sha.push_str("-dirty");
    }

    Some(sha)
}

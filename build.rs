use std::process::Command;

fn main() {
    let git_hash = get_git_hash().unwrap_or_else(|| String::from("unknown"));

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}

fn get_git_hash() -> Option<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .ok()?;

    String::from_utf8(output.stdout).ok()
}

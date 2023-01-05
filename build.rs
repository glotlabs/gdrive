use std::fs;
use std::path::PathBuf;

fn main() {
    let git_hash = read_git_hash().unwrap_or_else(|| String::from("unknown"));

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}

fn read_git_hash() -> Option<String> {
    let git_base_path = PathBuf::from(".git");

    let head_file_path = git_base_path.join("HEAD");
    let head_content = fs::read_to_string(head_file_path).ok()?;

    let head_ref = head_content.strip_prefix("ref: ")?;
    let head_ref_path = git_base_path.join(head_ref.trim());
    fs::read_to_string(head_ref_path).ok()
}

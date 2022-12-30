use std::env::consts;

pub fn version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("Rust: {}", rustc_version_runtime::version());
    println!("Arch: {}", consts::ARCH);
    println!("OS: {}", consts::OS);
}

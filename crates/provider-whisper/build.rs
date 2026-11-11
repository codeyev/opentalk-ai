use std::process::Command;

fn main() {
    // The Whisper engine's Metal code checks the macOS version at run time
    // (`@available`). When the app targets an older macOS than the one it
    // is built on, those checks call a helper from the compiler's runtime
    // library, which Rust does not link by default. Without it the release
    // build fails with "undefined symbol ___isPlatformVersionAtLeast".
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let runtime_dir = Command::new("xcrun")
        .args(["clang", "--print-runtime-dir"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok());
    if let Some(directory) = runtime_dir {
        println!("cargo:rustc-link-search=native={}", directory.trim());
        println!("cargo:rustc-link-lib=static=clang_rt.osx");
    }
}

fn main() {
    // ScreenCaptureKit is reached through a Swift bridge, so every macOS
    // binary that links this crate must be able to find the Swift runtime
    // that ships with the operating system. Cargo does not pass link
    // arguments on to dependent crates, so the desktop app's build script
    // repeats this line.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    }
}

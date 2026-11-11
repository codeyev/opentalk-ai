/// Every IPC command the app exposes. Tauri generates one permission per
/// command (`allow-app-info`, ...), and a command can only be called from a
/// window whose capability file grants that permission.
const COMMANDS: &[&str] = &[
    "app_info",
    "list_microphones",
    "start_capture",
    "stop_capture",
    "is_capturing",
    "transcription_provider",
    "translation_provider",
    "reply_provider",
    "refresh_replies",
    "ollama_status",
    "custom_provider",
    "whisper_models",
    "download_whisper_model",
    "cancel_model_download",
    "save_api_key",
    "delete_api_key",
    "has_api_key",
    "synthesize_speech",
];

fn main() {
    // The system-audio adapter uses a Swift bridge; see crates/audio-capture/build.rs.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    }

    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS));
    tauri_build::try_build(attributes).expect("failed to run the Tauri build script");
}

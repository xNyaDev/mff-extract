fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        tauri_winres::WindowsResource::new().compile().unwrap();
    }
}

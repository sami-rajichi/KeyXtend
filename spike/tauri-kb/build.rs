//! Embeds the app manifest: uiAccess only in release builds, plus Tauri's resources.

/// Manifest template beside this file.
const MANIFEST: &str = "app.manifest";
/// Placeholder in the template for the uiAccess value.
const SLOT: &str = "@UIACCESS@";
/// Cargo profile that gets uiAccess.
const RELEASE: &str = "release";

fn main() {
    println!("cargo:rerun-if-changed={MANIFEST}");
    println!("cargo:rerun-if-changed=ui");
    let release = std::env::var("PROFILE").is_ok_and(|p| p == RELEASE);
    let template = std::fs::read_to_string(MANIFEST).expect("app.manifest is readable");
    let manifest = template.replace(SLOT, if release { "true" } else { "false" });
    let windows = tauri_build::WindowsAttributes::new().app_manifest(manifest);
    let attributes = tauri_build::Attributes::new().windows_attributes(windows);
    if let Err(err) = tauri_build::try_build(attributes) {
        panic!("tauri-build failed: {err:#}");
    }
}

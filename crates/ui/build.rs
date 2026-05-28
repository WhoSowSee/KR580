//! Build script that embeds the Windows resource section into the binary so
//! Explorer, the taskbar, and Start show our icon for the `.exe` itself.
//! All the runtime icon plumbing (window/Alt-Tab) is handled at runtime via
//! iced's `window::Settings::icon`; this script is purely about the static
//! PE resource that Windows reads before our process even starts.

#[cfg(windows)]
fn main() {
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    // `crates/ui` -> workspace root.
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root must be two levels above crates/ui");
    let icon_path = workspace_root.join("assets").join("icons").join("icon.ico");

    println!("cargo:rerun-if-changed={}", icon_path.display());

    if !icon_path.exists() {
        // Don't break the build if the icon is missing — fresh
        // checkouts must compile before the artist has run the
        // icon script.
        println!(
            "cargo:warning=icon resource not embedded: {} is missing",
            icon_path.display()
        );
        return;
    }

    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(
        icon_path
            .to_str()
            .expect("icon path must be valid UTF-8 for the resource compiler"),
    );

    if let Err(error) = resource.compile() {
        // Surface as a warning rather than fail compilation —
        // cosmetic feature, too aggressive to block the build.
        println!("cargo:warning=failed to embed Windows icon resource: {error}");
    }
}

#[cfg(not(windows))]
fn main() {}

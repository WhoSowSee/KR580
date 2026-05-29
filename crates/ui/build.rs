//! Build script that embeds the Windows resource section into the binary so
//! Explorer, the taskbar, and Start show our icon for the `.exe` itself.
//! All the runtime icon plumbing (window/Alt-Tab) is handled at runtime via
//! iced's `window::Settings::icon`; this script is purely about the static
//! PE resource that Windows reads before our process even starts.

#[cfg(windows)]
fn main() {
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root must be two levels above crates/ui");
    let icons_dir = workspace_root.join("assets").join("icons");
    let icon_path = icons_dir.join("icon.ico");
    let file_icon_path = icons_dir.join("file-580.ico");

    println!("cargo:rerun-if-changed={}", icon_path.display());
    println!("cargo:rerun-if-changed={}", file_icon_path.display());

    if !icon_path.exists() {
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

    if file_icon_path.exists() {
        resource.set_icon_with_id(
            file_icon_path
                .to_str()
                .expect("file-icon path must be valid UTF-8 for the resource compiler"),
            "2",
        );
    } else {
        println!(
            "cargo:warning=file-type icon not embedded: {} is missing",
            file_icon_path.display()
        );
    }

    if let Err(error) = resource.compile() {
        println!("cargo:warning=failed to embed Windows icon resource: {error}");
    }
}

#[cfg(not(windows))]
fn main() {}

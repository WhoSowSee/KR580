//! Build script that embeds the Windows resource section into the binary so
//! Explorer, the taskbar, and Start show our icon for the `.exe` itself.
//! All the runtime icon plumbing (window/Alt-Tab) is handled at runtime via
//! iced's `window::Settings::icon`; this script is purely about the static
//! PE resource that Windows reads before our process even starts.

fn main() {
    write_installer_payload_module();
    embed_windows_resources();
}

fn write_installer_payload_module() {
    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let payload_dir = std::env::var_os("KR580_INSTALLER_PAYLOAD_DIR").map(std::path::PathBuf::from);
    println!("cargo:rerun-if-env-changed=KR580_INSTALLER_PAYLOAD_DIR");

    let code = match payload_dir {
        Some(dir) => {
            let kr = dir.join(binary_name("kr"));
            let k580 = dir.join(binary_name("k580"));
            let uninstaller = dir.join(binary_name("k580-uninstaller"));
            if !kr.is_file() || !k580.is_file() || !uninstaller.is_file() {
                panic!(
                    "installer payload missing: {}, {}, or {}",
                    kr.display(),
                    k580.display(),
                    uninstaller.display()
                );
            }
            println!("cargo:rerun-if-changed={}", kr.display());
            println!("cargo:rerun-if-changed={}", k580.display());
            println!("cargo:rerun-if-changed={}", uninstaller.display());
            format!(
                "pub const EMBEDDED_KR: Option<&'static [u8]> = Some(include_bytes!(r#\"{}\"#));\n\
                 pub const EMBEDDED_K580: Option<&'static [u8]> = Some(include_bytes!(r#\"{}\"#));\n\
                 pub const EMBEDDED_UNINSTALLER: Option<&'static [u8]> = Some(include_bytes!(r#\"{}\"#));\n",
                kr.display(),
                k580.display(),
                uninstaller.display()
            )
        }
        None => "pub const EMBEDDED_KR: Option<&'static [u8]> = None;\n\
             pub const EMBEDDED_K580: Option<&'static [u8]> = None;\n\
             pub const EMBEDDED_UNINSTALLER: Option<&'static [u8]> = None;\n"
            .to_owned(),
    };

    std::fs::write(out_dir.join("installer_payload.rs"), code)
        .expect("installer payload module must be writable");
}

fn binary_name(name: &str) -> String {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
}

#[cfg(windows)]
fn embed_windows_resources() {
    let manifest_dir = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let icons_dir = manifest_dir.join("assets").join("icons");
    let icon_path = windows_main_icon(&icons_dir);
    let file_icon_path = icons_dir.join("file-580.ico");

    println!("cargo:rerun-if-env-changed=KR580_WINDOWS_ICON_KIND");
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

#[cfg(windows)]
fn windows_main_icon(icons_dir: &std::path::Path) -> std::path::PathBuf {
    let icon_name = match std::env::var("KR580_WINDOWS_ICON_KIND").as_deref() {
        Ok("setup") => "installer-setup.ico",
        Ok("uninstaller") => "installer-uninstall.ico",
        _ => "icon.ico",
    };
    icons_dir.join(icon_name)
}

#[cfg(not(windows))]
fn embed_windows_resources() {}

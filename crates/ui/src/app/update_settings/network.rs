use super::super::network::{NetworkEndpointError, parse_network_endpoint};
use super::super::settings_modal::SettingsDialog;

pub(super) type NetworkDefaults = ((String, u16), (String, u16));

pub(super) fn parse_network_defaults(
    dialog: &SettingsDialog,
) -> Result<NetworkDefaults, NetworkEndpointError> {
    let client = parse_network_endpoint(
        &dialog.draft_network_client_host,
        &dialog.draft_network_client_port,
    )?;
    let server = parse_network_endpoint(
        &dialog.draft_network_server_host,
        &dialog.draft_network_server_port,
    )?;
    Ok((client, server))
}

pub(super) fn apply_network_defaults(
    settings: &mut k580_persistence::NetworkSettings,
    ((client_host, client_port), (server_host, server_port)): NetworkDefaults,
) {
    settings.host = client_host;
    settings.port = client_port;
    settings.bind_host = server_host;
    settings.bind_port = server_port;
}

pub(super) fn is_directory_writable(path: &std::path::Path) -> bool {
    internal_is_directory_writable(path)
}

#[cfg(unix)]
fn internal_is_directory_writable(path: &std::path::Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let mut buf = path.as_os_str().as_bytes().to_vec();
    buf.push(0);
    unsafe { libc::access(buf.as_ptr() as *const libc::c_char, libc::W_OK) == 0 }
}

#[cfg(windows)]
fn internal_is_directory_writable(path: &std::path::Path) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let test_file = path.join(format!(".kr580_{stamp:x}"));
    let ok = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&test_file)
        .is_ok();
    let _ = std::fs::remove_file(&test_file);
    ok
}

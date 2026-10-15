use semver::Version;
use serde::Deserialize;
use std::time::Duration;

const REPOSITORY: &str = "https://github.com/WhoSowSee/KR580";

#[derive(Clone, Debug)]
pub(crate) struct Release {
    pub(crate) version: String,
    pub(crate) page: String,
    pub(crate) asset: Option<Asset>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Asset {
    pub(crate) name: String,
    #[serde(rename = "browser_download_url")]
    pub(crate) url: String,
    pub(crate) size: u64,
    pub(crate) digest: Option<String>,
}

#[derive(Deserialize)]
struct ApiRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

pub(crate) async fn check() -> Result<Option<Release>, String> {
    let client = super::client()?;
    let mut response = client
        .get("https://api.github.com/repos/WhoSowSee/KR580/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        if bytes.len() + chunk.len() > 1024 * 1024 {
            return Err("release response is too large".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    parse(
        &bytes,
        env!("CARGO_PKG_VERSION"),
        env!("KR580_BUILD_TARGET"),
        install_supported(),
    )
}

fn install_supported() -> bool {
    if std::env::var_os("SNAP").is_some() || crate::package_kind::is_managed() {
        return false;
    }
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    if exe.starts_with("/nix/store") {
        return false;
    }
    if cfg!(target_os = "macos") {
        return exe
            .parent()
            .and_then(|path| path.parent())
            .is_some_and(|path| path.file_name().is_some_and(|name| name == "Contents"));
    }
    k580_ui::install_mode::manifest_for_executable(&exe)
        .ok()
        .flatten()
        .is_some_and(|(_, manifest)| manifest.manifest_version == 1)
}

fn parse(
    bytes: &[u8],
    current: &str,
    target: &str,
    supported: bool,
) -> Result<Option<Release>, String> {
    let release: ApiRelease = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    let version_text = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let version = Version::parse(version_text).map_err(|error| error.to_string())?;
    let current = Version::parse(current).map_err(|error| error.to_string())?;
    if release.draft || release.prerelease || !version.pre.is_empty() {
        return Err("latest release is not a stable published version".into());
    }
    if !version.cmp_precedence(&current).is_gt() {
        return Ok(None);
    }
    let name = if target.contains("apple-darwin") {
        format!("KR580-{version_text}-{target}.dmg")
    } else {
        format!("KR580-Setup-{}-{target}.zip", release.tag_name)
    };
    let prefix = format!("{REPOSITORY}/releases/download/{}/", release.tag_name);
    let asset = if supported {
        release.assets.into_iter().find(|asset| asset.name == name)
    } else {
        None
    };
    if let Some(asset) = &asset
        && (asset.size == 0
            || asset.size > 1024 * 1024 * 1024
            || asset.url != format!("{prefix}{}", asset.name)
            || !asset.digest.as_ref().is_some_and(|digest| {
                digest.starts_with("sha256:")
                    && digest.len() == 71
                    && digest[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
            }))
    {
        return Err("release asset metadata is invalid or has no SHA-256 digest".into());
    }
    Ok(Some(Release {
        version: version.to_string(),
        page: format!("{REPOSITORY}/releases/tag/{}", release.tag_name),
        asset,
    }))
}

#[cfg(test)]
mod tests;

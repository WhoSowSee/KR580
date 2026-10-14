use super::{check, parse};

fn response(tag: &str, prerelease: bool, url: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "tag_name": tag, "draft": false, "prerelease": prerelease,
        "assets": [{ "name": format!("KR580-Setup-{tag}-x86_64-pc-windows-msvc.zip"),
            "browser_download_url": url, "size": 16,
            "digest": format!("sha256:{}", "0".repeat(64)) }]
    }))
    .unwrap()
}

#[test]
fn release_comparison_uses_semver_and_ignores_build_metadata() {
    let bytes = response(
        "v4.10.0",
        false,
        "https://github.com/WhoSowSee/KR580/releases/download/v4.10.0/KR580-Setup-v4.10.0-x86_64-pc-windows-msvc.zip",
    );
    assert!(
        parse(&bytes, "4.9.0", "x86_64-pc-windows-msvc", true)
            .unwrap()
            .is_some()
    );
    assert!(
        parse(&bytes, "4.10.0+local", "x86_64-pc-windows-msvc", true)
            .unwrap()
            .is_none()
    );
    assert!(
        parse(&bytes, "5.0.0", "x86_64-pc-windows-msvc", true)
            .unwrap()
            .is_none()
    );
    assert!(
        parse(
            &response("v4.11.0-beta.1", true, "ignored"),
            "4.10.0",
            "target",
            false
        )
        .is_err()
    );
    assert!(parse(b"invalid json", "4.0.0", "target", false).is_err());
}

#[test]
fn assets_match_the_target_and_require_a_trusted_url_and_digest() {
    let url = "https://github.com/WhoSowSee/KR580/releases/download/v4.1.0/KR580-Setup-v4.1.0-x86_64-pc-windows-msvc.zip";
    let bytes = response("v4.1.0", false, url);
    assert!(
        parse(&bytes, "4.0.0", "x86_64-pc-windows-msvc", true)
            .unwrap()
            .unwrap()
            .asset
            .is_some()
    );
    assert!(
        parse(&bytes, "4.0.0", "aarch64-pc-windows-msvc", true)
            .unwrap()
            .unwrap()
            .asset
            .is_none()
    );
    assert!(
        parse(&bytes, "4.0.0", "x86_64-pc-windows-msvc", false)
            .unwrap()
            .unwrap()
            .asset
            .is_none()
    );
    assert!(
        parse(
            &response("v4.1.0", false, "https://example.com/setup.exe"),
            "4.0.0",
            "x86_64-pc-windows-msvc",
            true
        )
        .is_err()
    );
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["assets"][0]["digest"] = serde_json::Value::Null;
    assert!(
        parse(
            &serde_json::to_vec(&value).unwrap(),
            "4.0.0",
            "x86_64-pc-windows-msvc",
            true
        )
        .is_err()
    );
}

#[tokio::test]
#[ignore = "requires access to the live GitHub API"]
async fn live_release_check_completes_with_a_valid_result() {
    if let Some(release) = check().await.unwrap() {
        assert!(
            release
                .page
                .starts_with("https://github.com/WhoSowSee/KR580/releases/tag/")
        );
        assert!(
            semver::Version::parse(&release.version)
                .unwrap()
                .cmp_precedence(&semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap())
                .is_gt()
        );
    }
}

use super::extract_installer;
use std::io::Write;
use zip::write::SimpleFileOptions;

#[test]
fn installer_archive_reads_deflate_and_rejects_escaping_or_ambiguous_entries() {
    let directory = tempfile::tempdir().unwrap();
    let filename = format!(
        "KR580-Setup-4.1.0-{}{}",
        env!("KR580_BUILD_TARGET"),
        if cfg!(windows) { ".exe" } else { "" }
    );
    let payload = b"embedded setup payload".repeat(100);
    for (index, entries, expected) in [
        (0, vec![format!("package/{filename}")], true),
        (1, vec![format!("../{filename}")], false),
        (
            2,
            vec![filename.clone(), format!("package/{filename}")],
            false,
        ),
    ] {
        let root = directory.path().join(index.to_string());
        std::fs::create_dir(&root).unwrap();
        let archive = root.join("setup.zip");
        let mut writer = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
        for entry in entries {
            writer
                .start_file(
                    entry,
                    SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Deflated)
                        .compression_level(Some(6)),
                )
                .unwrap();
            writer.write_all(&payload).unwrap();
        }
        writer.finish().unwrap();
        let result = extract_installer(&archive, &root, "4.1.0");
        assert_eq!(result.is_ok(), expected);
        if expected {
            assert_eq!(std::fs::read(result.unwrap()).unwrap(), payload);
        } else {
            assert!(
                !root
                    .join(if cfg!(windows) {
                        "KR580-Setup.exe"
                    } else {
                        "KR580-Setup"
                    })
                    .exists()
            );
        }
    }
}

#[test]
#[ignore = "requires a downloaded official release ZIP"]
fn official_release_zip_extracts_installer() {
    let archive = std::env::var_os("KR580_UPDATE_TEST_ARCHIVE").expect("official release ZIP path");
    let version = std::env::var("KR580_UPDATE_TEST_VERSION").expect("official release version");
    let directory = tempfile::tempdir().unwrap();
    let helper =
        extract_installer(std::path::Path::new(&archive), directory.path(), &version).unwrap();
    assert!(std::fs::metadata(helper).unwrap().len() > 0);
}

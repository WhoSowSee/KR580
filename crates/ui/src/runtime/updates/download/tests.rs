use super::receive;
use crate::runtime::updates::{Asset, TransferProgress, ensure_tls_provider};
use std::io::{Read, Write};
use std::sync::atomic::Ordering;

#[tokio::test]
async fn streamed_download_validates_size_digest_and_progress() {
    ensure_tls_provider();
    let directory = tempfile::tempdir().unwrap();
    let bytes = b"a verified update fixture";
    let verified_digest = "sha256:f36290a67e59123e8e93ce630593770f687a296d70936a40475e6f48352fcd55";
    for (index, size, digest, expected) in [
        (0, bytes.len() as u64, verified_digest.to_owned(), true),
        (1, bytes.len() as u64 + 1, verified_digest.to_owned(), false),
        (
            2,
            bytes.len() as u64,
            format!("sha256:{}", "0".repeat(64)),
            false,
        ),
    ] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request).unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
                )
                .unwrap();
            for chunk in bytes.chunks(7) {
                write!(stream, "{:x}\r\n", chunk.len()).unwrap();
                stream.write_all(chunk).unwrap();
                stream.write_all(b"\r\n").unwrap();
            }
            stream.write_all(b"0\r\n\r\n").unwrap();
        });
        let response = reqwest::Client::new()
            .get(format!("http://{address}"))
            .send()
            .await
            .unwrap();
        let asset = Asset {
            name: "fixture.zip".into(),
            url: String::new(),
            size,
            digest: Some(digest),
        };
        let progress = TransferProgress::default();
        let path = directory.path().join(format!("{index}.zip"));
        assert_eq!(
            receive(response, &path, &asset, &progress).await.is_ok(),
            expected
        );
        assert_eq!(
            progress.received.load(Ordering::Relaxed),
            bytes.len() as u64
        );
        server.join().unwrap();
    }
}

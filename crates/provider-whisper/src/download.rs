//! Downloading a model file, when the user asks for one.
//!
//! Nothing here runs on its own. A download is written next to its final
//! place under a temporary name and only becomes a model once its size and
//! SHA-256 match the catalogue, so a partial or altered file is never used.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::models::ModelSpec;

/// Longest wait to reach the download server.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// Longest wait for more data before a stalled download is given up.
const READ_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum DownloadError {
    #[error("the download server could not be reached")]
    ConnectionFailed,
    #[error("the download was cancelled")]
    Cancelled,
    #[error("the downloaded file does not match the published model and was discarded")]
    VerificationFailed,
    #[error("the model could not be saved to disk")]
    StorageFailed,
}

/// Downloads `spec` from `url` into `directory` and returns the path of
/// the verified model. `on_progress` is called with the bytes received so
/// far; setting `cancelled` stops the download.
///
/// # Errors
///
/// Returns an error if the server cannot be reached, the download is
/// cancelled, the file fails verification, or it cannot be written. No
/// partial file is left behind in any of these cases.
pub async fn download(
    spec: &ModelSpec,
    url: &str,
    directory: &Path,
    cancelled: &AtomicBool,
    mut on_progress: impl FnMut(u64),
) -> Result<PathBuf, DownloadError> {
    fs::create_dir_all(directory)
        .await
        .map_err(|_| DownloadError::StorageFailed)?;
    let partial = directory.join(format!("{}.part", spec.file_name));
    let result = fetch(spec, url, &partial, cancelled, &mut on_progress).await;
    if result.is_err() {
        let _ = fs::remove_file(&partial).await;
        return result.map(|()| partial);
    }
    let finished = directory.join(spec.file_name);
    fs::rename(&partial, &finished)
        .await
        .map_err(|_| DownloadError::StorageFailed)?;
    Ok(finished)
}

/// Streams the file to `partial` and checks it against the catalogue.
async fn fetch(
    spec: &ModelSpec,
    url: &str,
    partial: &Path,
    cancelled: &AtomicBool,
    on_progress: &mut impl FnMut(u64),
) -> Result<(), DownloadError> {
    let client = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .build()
        .map_err(|_| DownloadError::ConnectionFailed)?;
    let mut response = client
        .get(url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|_| DownloadError::ConnectionFailed)?;
    let mut file = fs::File::create(partial)
        .await
        .map_err(|_| DownloadError::StorageFailed)?;
    let mut hasher = Sha256::new();
    let mut received: u64 = 0;

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| DownloadError::ConnectionFailed)?
    {
        if cancelled.load(Ordering::Relaxed) {
            return Err(DownloadError::Cancelled);
        }
        received += chunk.len() as u64;
        // A server sending more than the model's size is sending something else.
        if received > spec.size_bytes {
            return Err(DownloadError::VerificationFailed);
        }
        hasher.update(&chunk);
        file.write_all(&chunk)
            .await
            .map_err(|_| DownloadError::StorageFailed)?;
        on_progress(received);
    }
    file.flush()
        .await
        .map_err(|_| DownloadError::StorageFailed)?;

    let digest: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if received == spec.size_bytes && digest == spec.sha256 {
        Ok(())
    } else {
        Err(DownloadError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use opentalk_test_support::{MockServer, Route};

    use super::*;
    use crate::models::testing::TempDir;

    const FILE_PATH: &str = "/ggml-test.bin";
    const CONTENT: &str = "lmgg-pretend-model";
    /// SHA-256 of `CONTENT`.
    const CONTENT_SHA256: &str = "9fbb0762bdf35a9fbc25ed5830e67907e52089ec446623627216ecb06cf29f23";

    fn spec(size_bytes: u64, sha256: &'static str) -> ModelSpec {
        ModelSpec {
            id: "test",
            file_name: "ggml-test.bin",
            size_bytes,
            sha256,
        }
    }

    async fn serve(body: &str) -> (MockServer, String) {
        let server = MockServer::start(vec![Route::new(FILE_PATH, 200, body)]).await;
        let url = format!("{}{FILE_PATH}", server.base_url());
        (server, url)
    }

    fn files_in(folder: &TempDir) -> Vec<String> {
        std::fs::read_dir(&folder.0)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect()
    }

    #[tokio::test]
    async fn a_verified_download_becomes_a_model() {
        let folder = TempDir::new("download-ok");
        let (_server, url) = serve(CONTENT).await;
        let mut progress = Vec::new();

        let spec = spec(CONTENT.len() as u64, CONTENT_SHA256);
        let path = download(&spec, &url, &folder.0, &AtomicBool::new(false), |bytes| {
            progress.push(bytes)
        })
        .await
        .unwrap();

        assert_eq!(std::fs::read_to_string(path).unwrap(), CONTENT);
        assert_eq!(files_in(&folder), vec!["ggml-test.bin"]);
        assert_eq!(progress.last(), Some(&(CONTENT.len() as u64)));
    }

    #[tokio::test]
    async fn a_file_with_the_wrong_hash_is_discarded() {
        let folder = TempDir::new("download-hash");
        let (_server, url) = serve("lmgg-tampered-model").await;
        let spec = spec(19, CONTENT_SHA256);

        let result = download(&spec, &url, &folder.0, &AtomicBool::new(false), |_| {}).await;
        assert_eq!(result, Err(DownloadError::VerificationFailed));
        assert!(files_in(&folder).is_empty());
    }

    #[tokio::test]
    async fn a_file_larger_than_published_is_discarded() {
        let folder = TempDir::new("download-size");
        let (_server, url) = serve(CONTENT).await;
        let result = download(
            &spec(4, CONTENT_SHA256),
            &url,
            &folder.0,
            &AtomicBool::new(false),
            |_| {},
        )
        .await;
        assert_eq!(result, Err(DownloadError::VerificationFailed));
        assert!(files_in(&folder).is_empty());
    }

    #[tokio::test]
    async fn a_cancelled_download_leaves_nothing_behind() {
        let folder = TempDir::new("download-cancel");
        let (_server, url) = serve(CONTENT).await;
        let spec = spec(CONTENT.len() as u64, CONTENT_SHA256);
        let result = download(&spec, &url, &folder.0, &AtomicBool::new(true), |_| {}).await;
        assert_eq!(result, Err(DownloadError::Cancelled));
        assert!(files_in(&folder).is_empty());
    }

    #[tokio::test]
    async fn a_missing_file_on_the_server_is_a_connection_failure() {
        let folder = TempDir::new("download-404");
        let server = MockServer::start(Vec::new()).await;
        let url = format!("{}{FILE_PATH}", server.base_url());
        let result = download(
            &spec(1, CONTENT_SHA256),
            &url,
            &folder.0,
            &AtomicBool::new(false),
            |_| {},
        )
        .await;
        assert_eq!(result, Err(DownloadError::ConnectionFailed));
        assert!(files_in(&folder).is_empty());
    }
}

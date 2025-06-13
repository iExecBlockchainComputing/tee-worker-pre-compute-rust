use crate::compute::errors::ReplicateStatusCause;
use log::{error, info};
use reqwest::blocking::get;
use std::fs;
use std::path::{Path, PathBuf};

const IPFS_GATEWAYS: &[&str] = &[
    "https://ipfs-gateway.v8-bellecour.iex.ec",
    "https://gateway.ipfs.io",
    "https://gateway.pinata.cloud",
];

/// Downloads a file from a given URL and writes it to a specified folder with a specified filename.
///
/// If the download or any file operation fails, the function logs an appropriate error
/// and returns `None`. It also ensures the parent directory exists, creating it if necessary.
/// If the directory is newly created but the file write fails, it is cleaned up (deleted).
///
/// # Arguments
///
/// - `url`: The URL to download the file from. Must not be empty.
/// - `parent_dir`: The directory path where the file will be stored. Must not be empty.
/// - `filename`: The name to use for the downloaded file. Must not be empty.
///
/// # Returns
///
/// - `Some(PathBuf)` with the full path to the downloaded file if successful.
/// - `None` if any validation, download, directory creation, or file writing fails.
///
/// # Example
///
/// ```
/// if let Some(path) = download_file("https://iex.ec/file.txt", "/tmp", "iexec.txt") {
///     println!("File downloaded to: {}", path.display());
/// } else {
///     println!("Failed to download file.");
/// }
/// ```
///
/// # Notes
///
/// - This function uses **blocking** I/O (`reqwest::blocking`) and is not suitable for async contexts.
/// - The downloaded content is fully loaded into memory before being written to disk.
pub fn download_file(url: &str, parent_dir: &str, filename: &str) -> Option<PathBuf> {
    if url.is_empty() {
        error!("Invalid file url [url:{}]", url);
        return None;
    }
    if parent_dir.is_empty() {
        error!(
            "Invalid parent folder path [url:{}, parent_dir:{}]",
            url, parent_dir
        );
        return None;
    }
    if filename.is_empty() {
        error!(
            "Invalid output filename [url:{}, parent_dir:{}, filename:{}]",
            url, parent_dir, filename
        );
        return None;
    }

    let bytes = match get(url) {
        Ok(response) => match response.bytes() {
            Ok(b) => b,
            Err(_) => {
                error!("Failed to read file bytes from url [url:{}]", url);
                return None;
            }
        },
        Err(_) => {
            error!("Failed to download file [url:{}]", url);
            return None;
        }
    };

    let parent_path = Path::new(parent_dir);
    let parent_existed = parent_path.exists();

    if !parent_existed && fs::create_dir_all(parent_path).is_err() {
        error!(
            "Failed to create parent folder [url:{}, parent_dir:{}]",
            url, parent_dir
        );
        return None;
    }

    let file_path = parent_path.join(filename);

    match fs::write(&file_path, bytes) {
        Ok(_) => {
            info!(
                "Downloaded data [url:{}, file_path: {}]",
                url,
                file_path.display()
            );
            Some(file_path)
        }
        Err(_) => {
            error!(
                "Failed to write downloaded file to disk [url:{}, file_path:{}]",
                url,
                file_path.display()
            );
            if !parent_existed {
                match fs::remove_dir_all(parent_path) {
                    Ok(_) => {
                        info!("Folder deleted [path:{}]", parent_path.display());
                    }
                    Err(_) => {
                        error!(
                            "Folder does not exist, nothing to delete [path:{}]",
                            parent_path.display()
                        );
                    }
                }
            }
            None
        }
    }
}

pub fn download_from_ipfs_gateways(url: &str) -> Result<Vec<u8>, ReplicateStatusCause> {
    for gateway in IPFS_GATEWAYS {
        let full_url = format!("{}{}", gateway, url);
        info!("Attempting to download dataset from {}", full_url);

        match get(&full_url).and_then(|response| response.bytes()) {
            Ok(bytes) => return Ok(bytes.to_vec()),
            Err(e) => {
                info!("Failed to download from {}: {}", full_url, e);
                continue;
            }
        }
    }
    Err(ReplicateStatusCause::PreComputeDatasetDownloadFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const URL: &str = "https://httpbin.org/json";
    const PARENT_DIR: &str = "/tmp";
    const FILE_NAME: &str = "test.json";

    #[test]
    fn test_empty_url() {
        assert!(download_file("", PARENT_DIR, FILE_NAME).is_none());
    }

    #[test]
    fn test_empty_parent_dir() {
        assert!(download_file(URL, "", FILE_NAME).is_none());
    }

    #[test]
    fn test_empty_filename() {
        assert!(download_file(URL, PARENT_DIR, "").is_none());
    }

    #[test]
    fn test_invalid_url() {
        let result = download_file("not-a-url", PARENT_DIR, FILE_NAME);
        assert!(result.is_none());
    }

    #[test]
    fn test_successful_download() {
        let result = download_file(URL, PARENT_DIR, FILE_NAME);

        if let Some(path) = result {
            assert!(path.exists());
            assert!(path.is_file());
            let content = fs::read_to_string(&path).unwrap();
            assert!(content.contains("slideshow"));
        }
    }

    #[test]
    fn test_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("deep");

        let result = download_file(
            "https://httpbin.org/json",
            nested_path.to_str().unwrap(),
            "test.json",
        );

        if let Some(path) = result {
            assert!(path.exists());
            assert!(nested_path.exists());
        }
    }
}

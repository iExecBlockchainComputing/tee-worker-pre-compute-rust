use log::{error, info};
use reqwest::blocking::get;
use std::fs;
use std::path::{Path, PathBuf};

/// Writes content to a file at the specified path, with proper error handling and logging.
///
/// This function handles the common pattern of writing data to a file with logging
/// and error handling.
///
/// # Arguments
///
/// * `content` - The content to write to the file
/// * `file_path` - The path where the file should be written
/// * `context` - A context string for logging (e.g., "url:https://iex.ec/file.txt" or "chainTaskId:0x123")
///
/// # Returns
///
/// * `Ok(())` if the file is successfully written
/// * `Err(())` if the write operation fails
///
/// # Example
///
/// ```
/// let content = b"Hello, world!";
/// let path = PathBuf::from("/tmp/test.txt");
/// if write_file(content, &path, "test context").is_ok() {
///     println!("File written successfully");
/// }
/// ```
pub fn write_file(content: &[u8], file_path: &Path, context: &str) -> Result<(), ()> {
    match fs::write(file_path, content) {
        Ok(_) => {
            info!(
                "File written successfully [context:{}, path:{}]",
                context,
                file_path.display()
            );
            Ok(())
        }
        Err(_) => {
            error!(
                "Failed to write file [context:{}, path:{}]",
                context,
                file_path.display()
            );
            Err(())
        }
    }
}

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

    let bytes = match download_from_url(url) {
        Some(b) => b,
        None => {
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

    if write_file(&bytes, &file_path, &format!("url:{}", url)).is_ok() {
        Some(file_path)
    } else {
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

/// Downloads the content from the given URL and returns it as a byte vector.
///
/// This function supports any HTTP/HTTPS URL, including IPFS gateway URLs.
/// It performs a blocking GET request and returns the full response body as bytes.
///
/// # Arguments
///
/// * `url` - The URL to download from. Must not be empty.
///
/// # Returns
///
/// * `Some(Vec<u8>)` if the download succeeds and the response body is read successfully.
/// * `None` if the URL is empty, the request fails, or the response status is not successful.
///
/// # Example
///
/// ```
/// if let Some(bytes) = download_from_url("https://httpbin.org/json/test.json") {
///     println!("Downloaded {} bytes", bytes.len());
/// } else {
///     println!("Download failed");
/// }
/// ```
///
/// # Notes
///
/// - This function uses blocking I/O and is not suitable for async contexts.
/// - The entire response body is loaded into memory.
pub fn download_from_url(url: &str) -> Option<Vec<u8>> {
    if url.is_empty() {
        error!("Invalid URL: empty string");
        return None;
    }

    info!("Attempting to download from {}", url);

    match get(url)
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.bytes())
    {
        Ok(bytes) => {
            info!("Successfully downloaded {} bytes from {}", bytes.len(), url);
            Some(bytes.to_vec())
        }
        Err(e) => {
            error!("Failed to download from {}: {}", url, e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::TempDir;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const URL: &str = "https://httpbin.org/json";
    const PARENT_DIR: &str = "/tmp";
    const FILE_NAME: &str = "test.json";

    // region download_file
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
    // endregion

    // region download_from_url
    #[test]
    fn test_download_from_url_success() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let expected_data = b"test data";

        let mock_server = rt.block_on(async {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/test"))
                .respond_with(ResponseTemplate::new(200).set_body_bytes(expected_data))
                .mount(&server)
                .await;
            server
        });

        let server_uri = mock_server.uri();
        let result = download_from_url(&format!("{}/test", server_uri));

        assert!(result.is_some());
        assert_eq!(result.unwrap(), expected_data);
    }

    #[test]
    fn test_download_from_url_with_empty_url() {
        let result = download_from_url("");
        assert!(result.is_none());
    }

    #[test]
    fn test_download_from_url_with_invalid_url() {
        let result = download_from_url("not-a-valid-url");
        assert!(result.is_none());
    }

    #[test]
    fn test_download_from_url_with_server_error() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let mock_server = rt.block_on(async {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/error"))
                .respond_with(ResponseTemplate::new(500))
                .mount(&server)
                .await;
            server
        });

        let server_uri = mock_server.uri();
        let result = download_from_url(&format!("{}/error", server_uri));

        assert!(result.is_none());
    }
    // endregion

    // region write_file
    #[test]
    fn test_write_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_write.txt");
        let content = b"hello world!";
        let context = "test_write_file_success";
        let result = write_file(content, &file_path, context);
        assert!(result.is_ok());
        let mut file = fs::File::open(&file_path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, content);
    }

    #[test]
    fn test_write_file_failure_invalid_path() {
        let file_path = Path::new("/invalid_dir_123456789/test.txt");
        let content = b"should fail";
        let context = "test_write_file_failure_invalid_path";
        let result = write_file(content, file_path, context);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_file_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("overwrite.txt");
        let context = "test_write_file_overwrite";
        let content1 = b"first";
        assert!(write_file(content1, &file_path, context).is_ok());
        let content2 = b"second";
        assert!(write_file(content2, &file_path, context).is_ok());
        let data = fs::read(&file_path).unwrap();
        assert_eq!(data, content2);
    }
    // endregion
}

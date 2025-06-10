use crate::compute::errors::ReplicateStatusCause;
use crate::compute::pre_compute_args::PreComputeArgs;
use crate::compute::utils::file_utils::download_file;
use crate::compute::utils::hash_utils::sha256;
use log::{error, info};
use mockall::automock;
use std::path::Path;

#[automock]
pub trait PreComputeAppTrait {
    fn run(&mut self, chain_task_id: &str) -> Result<(), ReplicateStatusCause>;
    fn check_output_folder(&self) -> Result<(), ReplicateStatusCause>;
    fn download_input_files(&self) -> Result<(), ReplicateStatusCause>;
}

pub struct PreComputeApp {
    chain_task_id: Option<String>,
    pre_compute_args: Option<PreComputeArgs>,
}

impl PreComputeApp {
    pub fn new() -> Self {
        PreComputeApp {
            chain_task_id: None,
            pre_compute_args: None,
        }
    }
}

impl PreComputeAppTrait for PreComputeApp {
    fn run(&mut self, chain_task_id: &str) -> Result<(), ReplicateStatusCause> {
        self.chain_task_id = Some(chain_task_id.to_string());
        self.pre_compute_args = Some(PreComputeArgs::read_args()?);
        self.check_output_folder()?;
        self.download_input_files()?;
        Ok(())
    }

    /// Checks whether the output folder specified in `pre_compute_args` exists.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the output directory (`output_dir`) exists.
    /// - `Err(ReplicateStatusCause::PreComputeOutputFolderNotFound)` if the directory does not exist,
    ///   or if `pre_compute_args` is missing.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::pre_compute_app::PreComputeApp;
    ///
    /// let pre_compute_app = PreComputeApp::new();
    /// pre_compute_app.chain_task_id = Some("0x123456789abcdef");
    /// pre_compute_app.pre_compute_args = Some(PreComputeArgs::read_args()?);
    ///
    /// pre_compute_app.check_output_folder()?;
    /// ```
    fn check_output_folder(&self) -> Result<(), ReplicateStatusCause> {
        let output_dir = self
            .pre_compute_args
            .as_ref()
            .ok_or(ReplicateStatusCause::PreComputeOutputFolderNotFound)?
            .output_dir
            .clone();

        let chain_task_id = self.chain_task_id.as_deref().unwrap_or("unknown");

        info!(
            "Checking output folder [chainTaskId:{}, path:{}]",
            chain_task_id, output_dir
        );

        if Path::new(&output_dir).is_dir() {
            return Ok(());
        }

        error!(
            "Output folder not found [chainTaskId:{}, path:{}]",
            chain_task_id, output_dir
        );

        Err(ReplicateStatusCause::PreComputeOutputFolderNotFound)
    }

    /// Downloads the input files listed in `pre_compute_args.input_files` to the specified `output_dir`.
    ///
    /// Each URL is hashed (SHA-256) to generate a unique local filename.
    /// If any download fails, the function returns an error.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all files are downloaded successfully.
    /// - `Err(ReplicateStatusCause::PreComputeInputFileDownloadFailed)` if any file fails to download.
    ///
    /// # Panics
    ///
    /// This function panics if:
    /// - `pre_compute_args` is `None`.
    /// - `chain_task_id` is `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::pre_compute_app::PreComputeApp;
    ///
    /// let pre_compute_app = PreComputeApp::new();
    /// pre_compute_app.chain_task_id = Some("0x123456789abcdef");
    /// pre_compute_app.pre_compute_args = Some(PreComputeArgs::read_args()?);
    ///
    /// pre_compute_app.download_input_files()?;
    /// ```
    fn download_input_files(&self) -> Result<(), ReplicateStatusCause> {
        let args = self.pre_compute_args.as_ref().unwrap();
        let chain_task_id = self.chain_task_id.as_ref().unwrap();

        for url in &args.input_files {
            info!(
                "Downloading input file [chainTaskId: {}, url: {}]",
                chain_task_id, url
            );

            let filename = sha256(url.to_string());
            if download_file(url, &args.output_dir, &filename).is_none() {
                return Err(ReplicateStatusCause::PreComputeInputFileDownloadFailed);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::pre_compute_args::PreComputeArgs;
    use tempfile::{TempDir, tempdir};

    const CHAIN_TASK_ID: &str = "0x123456789abcdef";

    fn get_pre_compute_app(
        chain_task_id: &str,
        urls: Vec<&str>,
        output_dir: &str,
    ) -> PreComputeApp {
        PreComputeApp {
            chain_task_id: Some(chain_task_id.to_string()),
            pre_compute_args: Some(PreComputeArgs {
                input_files: urls.into_iter().map(String::from).collect(),
                output_dir: output_dir.to_string(),
                is_dataset_required: false,
                encrypted_dataset_url: None,
                encrypted_dataset_base64_key: None,
                encrypted_dataset_checksum: None,
                plain_dataset_filename: None,
            }),
        }
    }

    #[test]
    fn check_output_folder_returns_ok_with_valid_args() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().to_str().unwrap();

        let app = get_pre_compute_app(CHAIN_TASK_ID, vec![], output_path);

        let result = app.check_output_folder();
        assert!(result.is_ok());
    }

    #[test]
    fn check_output_folder_returns_err_with_invalid_file_path() {
        let non_existing_path = "/tmp/some_non_existing_output_dir_xyz_123".to_string();

        let app = get_pre_compute_app(CHAIN_TASK_ID, vec![], &non_existing_path);

        let result = app.check_output_folder();
        assert_eq!(
            result,
            Err(ReplicateStatusCause::PreComputeOutputFolderNotFound)
        );
    }

    #[test]
    fn check_output_folder_returns_err_with_invalid_pre_compute_args() {
        let app = PreComputeApp {
            chain_task_id: Some(CHAIN_TASK_ID.to_string()),
            pre_compute_args: None,
        };

        let result = app.check_output_folder();
        assert_eq!(
            result,
            Err(ReplicateStatusCause::PreComputeOutputFolderNotFound)
        );
    }

    #[test]
    fn download_input_files_success_with_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let app = get_pre_compute_app(
            CHAIN_TASK_ID,
            vec!["https://httpbin.org/json"],
            temp_dir.path().to_str().unwrap(),
        );

        let result = app.download_input_files();
        assert!(result.is_ok());

        let url_hash = sha256("https://httpbin.org/json".to_string());
        let downloaded_file = temp_dir.path().join(url_hash);
        assert!(downloaded_file.exists());
    }

    #[test]
    fn download_input_files_success_with_single_file_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let app = get_pre_compute_app(
            CHAIN_TASK_ID,
            vec!["https://httpbin.org/json", "https://httpbin.org/xml"],
            temp_dir.path().to_str().unwrap(),
        );

        let result = app.download_input_files();
        assert!(result.is_ok());

        let json_hash = sha256("https://httpbin.org/json".to_string());
        let xml_hash = sha256("https://httpbin.org/xml".to_string());

        assert!(temp_dir.path().join(json_hash).exists());
        assert!(temp_dir.path().join(xml_hash).exists());
    }

    #[test]
    fn test_download_failure_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let app = get_pre_compute_app(
            CHAIN_TASK_ID,
            vec!["https://invalid-url-that-should-fail.com/file.txt"],
            temp_dir.path().to_str().unwrap(),
        );

        let result = app.download_input_files();
        assert_eq!(
            result.unwrap_err(),
            ReplicateStatusCause::PreComputeInputFileDownloadFailed
        );
    }

    #[test]
    fn test_partial_failure_stops_on_first_error() {
        let temp_dir = TempDir::new().unwrap();
        let app = get_pre_compute_app(
            CHAIN_TASK_ID,
            vec![
                "https://httpbin.org/json",                          // This should succeed
                "https://invalid-url-that-should-fail.com/file.txt", // This should fail
                "https://httpbin.org/xml",                           // This shouldn't be reached
            ],
            temp_dir.path().to_str().unwrap(),
        );

        let result = app.download_input_files();
        assert_eq!(
            result.unwrap_err(),
            ReplicateStatusCause::PreComputeInputFileDownloadFailed
        );

        // First file should be downloaded with SHA256 filename
        let json_hash = sha256("https://httpbin.org/json".to_string());
        assert!(temp_dir.path().join(json_hash).exists());

        // Third file should NOT be downloaded (stopped on second failure)
        let xml_hash = sha256("https://httpbin.org/xml".to_string());
        assert!(!temp_dir.path().join(xml_hash).exists());
    }
}

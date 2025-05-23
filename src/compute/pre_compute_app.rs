use crate::compute::errors::ReplicateStatusCause;
use crate::compute::pre_compute_args::PreComputeArgs;
use log::{error, info};
use mockall::automock;
use std::path::Path;

#[automock]
pub trait PreComputeAppTrait {
    fn run(&mut self, chain_task_id: &str) -> Result<(), ReplicateStatusCause>;
    fn check_output_folder(&self) -> Result<(), ReplicateStatusCause>;
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
        Ok(())
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::pre_compute_args::PreComputeArgs;
    use tempfile::tempdir;

    const CHAIN_TASK_ID: &str = "0x123456789abcdef";

    #[test]
    fn check_output_folder_returns_ok_with_valid_args() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().to_str().unwrap().to_string();

        let app = PreComputeApp {
            chain_task_id: Some(CHAIN_TASK_ID.to_string()),
            pre_compute_args: Some(PreComputeArgs {
                output_dir: output_path,
                is_dataset_required: false,
                encrypted_dataset_url: None,
                encrypted_dataset_base64_key: None,
                encrypted_dataset_checksum: None,
                plain_dataset_filename: None,
                input_files: vec![],
            }),
        };

        let result = app.check_output_folder();
        assert!(result.is_ok());
    }

    #[test]
    fn check_output_folder_returns_err_with_invalid_file_path() {
        let non_existing_path = "/tmp/some_non_existing_output_dir_xyz_123".to_string();

        let app = PreComputeApp {
            chain_task_id: Some(CHAIN_TASK_ID.to_string()),
            pre_compute_args: Some(PreComputeArgs {
                output_dir: non_existing_path,
                is_dataset_required: false,
                encrypted_dataset_url: None,
                encrypted_dataset_base64_key: None,
                encrypted_dataset_checksum: None,
                plain_dataset_filename: None,
                input_files: vec![],
            }),
        };

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
}

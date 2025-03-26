use bytes::Bytes;
use std::path::{Path, PathBuf};
use std::{
    fs::{self, File},
    io::{self, Cursor},
    time::SystemTime,
};
use zip::result::ZipError;
use zip::ZipArchive;

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Failed to download the file from {0}")]
    DownloadFailed(String),
    #[error("Failed to write to output path: {0}")]
    IoError(#[from] io::Error),
    #[error("Failed to extract zip file")]
    ZipError(#[from] ZipError),
    #[error("Failed to create tarball: {0}")]
    TarError(String),
    #[error("Failed to delete files in {0}")]
    RemoveFailed(String),
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
}

#[cfg(unix)]
pub struct UnixResourceManager {
    tmp_path: PathBuf,
    agent_path: PathBuf,
}

#[trait_variant::make(Send)]
pub trait ResourceManagerTrait: Send + Sync {
    fn backup(&self) -> Result<(), ResourceError>;

    fn rollback(&self, backup_file: &Path) -> Result<(), ResourceError>;

    fn tmp_path(&self) -> &PathBuf;

    fn agent_path(&self) -> &PathBuf;

    async fn download_update_resources(
        &self,
        binary_url: &str,
        output_path: Option<impl AsRef<Path> + Send>,
    ) -> Result<(), ResourceError> {
        async move {
            let output_path = output_path.map(|x| x.as_ref().to_path_buf());
            let download_path = output_path.as_ref().unwrap_or(self.tmp_path());

            let response = reqwest::get(binary_url)
                .await
                .map_err(|_| ResourceError::DownloadFailed(binary_url.to_string()))?;
            let content = response
                .bytes()
                .await
                .map_err(|_| ResourceError::DownloadFailed(binary_url.to_string()))?;

            self.extract_zip(content, download_path)?;
            Ok(())
        }
    }

    fn get_latest_backup(&self) -> Option<PathBuf> {
        fs::read_dir(self.tmp_path())
            .ok()?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("gz")
            })
            .max_by_key(|path| {
                path.metadata()
                    .and_then(|meta| meta.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            })
    }

    fn extract_zip(&self, archive_data: Bytes, output_path: &Path) -> Result<(), ResourceError> {
        let cursor = Cursor::new(archive_data);
        let mut archive = ZipArchive::new(cursor)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = output_path.join(file.mangled_name());

            if file.is_file() {
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let _ = fs::remove_file(&file_path);
                let mut output_file = File::create(&file_path)?;
                io::copy(&mut file, &mut output_file)?;
                #[cfg(unix)]
                if let Some(file_name) = file_path.file_name() {
                    if file_name == "miax-agent" {
                        crate::unix_utils::change_to_executable(&file_path)?;
                    }
                }
            } else if file.is_dir() {
                fs::create_dir_all(&file_path)?;
            }
        }

        Ok(())
    }

    fn remove_directory(&self, path: &Path) -> Result<(), io::Error> {
        if !path.exists() {
            return Ok(());
        }

        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("Failed to remove directory {:?}: {}", path, e),
                )
            })?;
        } else {
            fs::remove_file(path).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("Failed to remove file {:?}: {}", path, e),
                )
            })?;
        }
        Ok(())
    }

    fn remove(&self) -> Result<(), ResourceError> {
        for entry in fs::read_dir(self.tmp_path())
            .map_err(|e| ResourceError::RemoveFailed(format!("Failed to read directory: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                ResourceError::RemoveFailed(format!("Failed to access entry: {}", e))
            })?;
            let entry_path = entry.path();

            self.remove_directory(&entry_path).map_err(|e| {
                ResourceError::RemoveFailed(format!(
                    "Failed to remove path {:?}: {}",
                    entry_path, e
                ))
            })?;
        }
        Ok(())
    }
}

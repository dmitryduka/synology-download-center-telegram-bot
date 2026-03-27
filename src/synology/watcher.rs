use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DropError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Watch folder does not exist: {0}")]
    FolderMissing(String),
}

/// Saves .torrent files to Download Station's watched folder.
#[derive(Clone)]
pub struct TorrentDropper {
    watch_folder: PathBuf,
}

impl TorrentDropper {
    pub fn new(watch_folder: &str) -> Self {
        Self {
            watch_folder: PathBuf::from(watch_folder),
        }
    }

    pub fn watch_folder(&self) -> &Path {
        &self.watch_folder
    }

    /// Save a .torrent file to the watch folder.
    /// Download Station will automatically pick it up.
    pub async fn drop_torrent(&self, filename: &str, data: &[u8]) -> Result<PathBuf, DropError> {
        if !self.watch_folder.exists() {
            return Err(DropError::FolderMissing(
                self.watch_folder.display().to_string(),
            ));
        }

        let safe_name = sanitize_filename(filename);
        let dest = self.watch_folder.join(&safe_name);

        tokio::fs::write(&dest, data).await?;

        log::info!("Torrent saved: {}", dest.display());
        Ok(dest)
    }
}

fn sanitize_filename(name: &str) -> String {
    let name = name.replace(['/', '\\', '\0'], "_");
    if name.is_empty() {
        "download.torrent".to_string()
    } else if !name.ends_with(".torrent") {
        format!("{}.torrent", name)
    } else {
        name
    }
}

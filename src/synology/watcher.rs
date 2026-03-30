use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct TorrentDropper {
    watch_folder: PathBuf,
}

impl TorrentDropper {
    pub fn new(watch_folder: &str) -> Self {
        Self { watch_folder: PathBuf::from(watch_folder) }
    }

    pub async fn drop_torrent(&self, filename: &str, data: &[u8]) -> Result<String, String> {
        if !self.watch_folder.exists() {
            return Err(format!("Watch folder does not exist: {}", self.watch_folder.display()));
        }
        let safe = sanitize(filename);
        let dest = self.watch_folder.join(&safe);
        tokio::fs::write(&dest, data).await.map_err(|e| format!("Write failed: {}", e))?;
        log::info!("Torrent saved: {}", dest.display());
        Ok(safe)
    }
}

fn sanitize(name: &str) -> String {
    let name = name.replace(['/', '\\', '\0'], "_");
    if name.is_empty() { "download.torrent".to_string() }
    else if !name.ends_with(".torrent") { format!("{}.torrent", name) }
    else { name }
}

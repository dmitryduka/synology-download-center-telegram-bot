use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

const DSM_BASE: &str = "https://localhost:5001/webapi/entry.cgi";
const CREDS_FILE: &str = "service_credentials.json";
const CREDS_DIRS: &[&str] = &["/volume1/watch", "/volume1/downloads", "/volume1/homes"];

#[derive(Debug, Deserialize, Clone)]
struct Credentials {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct SynoResp<T> {
    success: bool,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct AuthData {
    sid: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskList {
    pub task: Vec<Task>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub size: i64,
    pub status: i64,
    pub additional: Option<TaskAdditional>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskAdditional {
    pub transfer: Option<TaskTransfer>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskTransfer {
    pub size_downloaded: Option<i64>,
    pub speed_download: Option<i64>,
}

#[derive(Clone)]
pub struct DsmApi {
    http: reqwest::Client,
    sid: Arc<RwLock<Option<String>>>,
}

impl DsmApi {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("http client");
        Self {
            http,
            sid: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn is_configured(&self) -> bool {
        self.load_credentials().await.is_some()
    }

    async fn load_credentials(&self) -> Option<Credentials> {
        for dir in CREDS_DIRS {
            let path = PathBuf::from(dir).join(CREDS_FILE);
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                if let Ok(creds) = serde_json::from_str::<Credentials>(&content) {
                    return Some(creds);
                }
            }
        }
        None
    }

    async fn ensure_session(&self) -> Result<String, String> {
        {
            let sid = self.sid.read().await;
            if let Some(ref s) = *sid {
                return Ok(s.clone());
            }
        }
        self.login().await
    }

    async fn login(&self) -> Result<String, String> {
        let creds = self.load_credentials().await
            .ok_or("No service credentials. Open DSM app and click 'Setup Bot Access'.")?;

        let url = format!(
            "{}?api=SYNO.API.Auth&version=6&method=login&account={}&passwd={}&session=DownloadStation&format=sid",
            DSM_BASE,
            urlencoding::encode(&creds.username),
            urlencoding::encode(&creds.password),
        );

        let resp = self.http.get(&url).send().await.map_err(|e| e.to_string())?;
        let body: SynoResp<AuthData> = resp.json().await.map_err(|e| e.to_string())?;

        if body.success {
            if let Some(data) = body.data {
                if let Some(sid) = data.sid {
                    let mut lock = self.sid.write().await;
                    *lock = Some(sid.clone());
                    log::info!("DSM API login successful");
                    return Ok(sid);
                }
            }
        }
        Err("Login failed — check service credentials".into())
    }

    async fn api_call<T: serde::de::DeserializeOwned>(&self, params: &str) -> Result<T, String> {
        let sid = self.ensure_session().await?;
        let url = format!("{}&_sid={}", params, sid);
        let resp = self.http.get(&url).send().await.map_err(|e| e.to_string())?;
        let body: SynoResp<T> = resp.json().await.map_err(|e| e.to_string())?;

        if body.success {
            return body.data.ok_or("Empty response".into());
        }

        // Session expired — retry once
        {
            let mut lock = self.sid.write().await;
            *lock = None;
        }
        let sid = self.login().await?;
        let url = format!("{}&_sid={}", params, sid);
        let resp = self.http.get(&url).send().await.map_err(|e| e.to_string())?;
        let body: SynoResp<T> = resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            body.data.ok_or("Empty response".into())
        } else {
            Err("API call failed after re-auth".into())
        }
    }

    pub async fn list_tasks(&self) -> Result<TaskList, String> {
        let params = format!(
            "{}?api=SYNO.DownloadStation2.Task&version=2&method=list&offset=0&limit=-1&additional=%5B%22detail%22%2C%22transfer%22%5D",
            DSM_BASE
        );
        self.api_call(&params).await
    }

    /// Find a recently added task by title substring and edit its destination.
    pub async fn set_task_destination(&self, title_hint: &str, destination: &str) -> Result<(), String> {
        // Wait a bit for DS to pick up the torrent from the watch folder
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let tasks = self.list_tasks().await?;
        let task = tasks.task.iter()
            .filter(|t| t.status == 1 || t.status == 2) // waiting or downloading
            .find(|t| t.title.contains(title_hint) || title_hint.contains(&t.title));

        if let Some(t) = task {
            let params = format!(
                "{}?api=SYNO.DownloadStation2.Task&version=2&method=edit&id=%5B%22{}%22%5D&destination=%22{}%22",
                DSM_BASE, t.id, urlencoding::encode(destination),
            );
            log::info!("Editing task {} destination to {}", t.id, destination);
            let _: serde_json::Value = self.api_call(&params).await?;
            Ok(())
        } else {
            log::warn!("Could not find task matching '{}' to edit destination", title_hint);
            Ok(()) // Don't error — the torrent was still added
        }
    }

    pub async fn pause_task(&self, id: &str) -> Result<(), String> {
        let params = format!(
            "{}?api=SYNO.DownloadStation2.Task&version=2&method=pause&id=%5B%22{}%22%5D",
            DSM_BASE, id
        );
        let _: serde_json::Value = self.api_call(&params).await?;
        Ok(())
    }

    pub async fn delete_task(&self, id: &str) -> Result<(), String> {
        let params = format!(
            "{}?api=SYNO.DownloadStation2.Task&version=2&method=delete&id=%5B%22{}%22%5D&force_complete=false",
            DSM_BASE, id
        );
        let _: serde_json::Value = self.api_call(&params).await?;
        Ok(())
    }
}

impl Task {
    pub fn progress(&self) -> u8 {
        if self.size <= 0 { return 0; }
        let dl = self.additional.as_ref()
            .and_then(|a| a.transfer.as_ref())
            .and_then(|t| t.size_downloaded)
            .unwrap_or(0);
        ((dl as f64 / self.size as f64) * 100.0).min(100.0) as u8
    }

    pub fn speed(&self) -> i64 {
        self.additional.as_ref()
            .and_then(|a| a.transfer.as_ref())
            .and_then(|t| t.speed_download)
            .unwrap_or(0)
    }

    pub fn status_name(&self) -> &str {
        match self.status {
            1 => "waiting", 2 => "downloading", 3 => "paused", 4 => "finishing",
            5 => "finished", 6 => "checking", 7 => "seeding", 8 => "waiting",
            9 => "extracting", 10 => "error", _ => "unknown",
        }
    }

    pub fn status_icon(&self) -> &str {
        match self.status {
            1 | 8 => "⏳", 2 => "⬇️", 3 => "⏸", 4 => "🏁",
            5 => "✅", 6 => "🔍", 7 => "⬆️", 9 => "📦", 10 => "❌", _ => "❓",
        }
    }

    pub fn is_finished(&self) -> bool { matches!(self.status, 5 | 7) }
    pub fn is_error(&self) -> bool { self.status == 10 }
    pub fn is_active(&self) -> bool { matches!(self.status, 1 | 2 | 6 | 8) }
    pub fn is_in_progress(&self) -> bool { matches!(self.status, 1 | 2 | 3 | 4 | 6 | 8 | 9) }
}

pub fn format_size(b: i64) -> String {
    if b >= 1_099_511_627_776 { format!("{:.1} TB", b as f64 / 1_099_511_627_776.0) }
    else if b >= 1_073_741_824 { format!("{:.1} GB", b as f64 / 1_073_741_824.0) }
    else if b >= 1_048_576 { format!("{:.1} MB", b as f64 / 1_048_576.0) }
    else if b >= 1024 { format!("{:.1} KB", b as f64 / 1024.0) }
    else { format!("{} B", b) }
}

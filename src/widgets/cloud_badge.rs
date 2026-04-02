use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Cloud sync status for a file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    Synced,
    Syncing,
    Offline,
    Error,
    Unknown,
}

impl SyncStatus {
    pub fn icon_name(&self) -> &'static str {
        match self {
            SyncStatus::Synced => "emblem-ok-symbolic",
            SyncStatus::Syncing => "emblem-synchronizing-symbolic",
            SyncStatus::Offline => "cloud-disabled-symbolic",
            SyncStatus::Error => "emblem-important-symbolic",
            SyncStatus::Unknown => "",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            SyncStatus::Synced => "sync-ok",
            SyncStatus::Syncing => "sync-progress",
            SyncStatus::Offline => "sync-offline",
            SyncStatus::Error => "sync-error",
            SyncStatus::Unknown => "",
        }
    }
}

/// Detects cloud sync providers and checks file status
pub struct CloudStatusProvider {
    providers: Vec<Box<dyn CloudProvider>>,
    cache: Arc<Mutex<HashMap<PathBuf, SyncStatus>>>,
}

trait CloudProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_managed(&self, path: &Path) -> bool;
    fn get_status(&self, path: &Path) -> SyncStatus;
}

/// Nextcloud sync client detection
struct NextcloudProvider {
    sync_dir: Option<PathBuf>,
}

impl NextcloudProvider {
    fn new() -> Self {
        // Nextcloud typically syncs to ~/Nextcloud
        let home = glib::home_dir();
        let sync_dir = home.join("Nextcloud");
        Self {
            sync_dir: if sync_dir.exists() { Some(sync_dir) } else { None },
        }
    }
}

impl CloudProvider for NextcloudProvider {
    fn name(&self) -> &str { "Nextcloud" }

    fn is_managed(&self, path: &Path) -> bool {
        self.sync_dir.as_ref().map_or(false, |sd| path.starts_with(sd))
    }

    fn get_status(&self, path: &Path) -> SyncStatus {
        if !self.is_managed(path) { return SyncStatus::Unknown; }

        // Check for Nextcloud .sync-exclude.lst or status database
        // Nextcloud stores status in ~/.local/share/data/Nextcloud/
        // For simplicity, check if the file exists and has recent mtime
        if let Ok(meta) = std::fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                let age = modified.elapsed().unwrap_or_default();
                if age.as_secs() < 30 {
                    return SyncStatus::Syncing;
                }
            }
            SyncStatus::Synced
        } else {
            SyncStatus::Error
        }
    }
}

/// Google Drive (via gvfs or insync/drive)
struct GDriveProvider {
    sync_dir: Option<PathBuf>,
}

impl GDriveProvider {
    fn new() -> Self {
        let home = glib::home_dir();
        // Common Google Drive sync directories
        let candidates = [
            home.join("Google Drive"),
            home.join("google-drive"),
            home.join(".insync"),
        ];
        let sync_dir = candidates.into_iter().find(|p| p.exists());
        Self { sync_dir }
    }
}

impl CloudProvider for GDriveProvider {
    fn name(&self) -> &str { "Google Drive" }

    fn is_managed(&self, path: &Path) -> bool {
        self.sync_dir.as_ref().map_or(false, |sd| path.starts_with(sd))
    }

    fn get_status(&self, path: &Path) -> SyncStatus {
        if !self.is_managed(path) { return SyncStatus::Unknown; }
        if path.exists() { SyncStatus::Synced } else { SyncStatus::Offline }
    }
}

/// Dropbox detection
struct DropboxProvider {
    sync_dir: Option<PathBuf>,
}

impl DropboxProvider {
    fn new() -> Self {
        let home = glib::home_dir();
        let sync_dir = home.join("Dropbox");
        Self {
            sync_dir: if sync_dir.exists() { Some(sync_dir) } else { None },
        }
    }
}

impl CloudProvider for DropboxProvider {
    fn name(&self) -> &str { "Dropbox" }

    fn is_managed(&self, path: &Path) -> bool {
        self.sync_dir.as_ref().map_or(false, |sd| path.starts_with(sd))
    }

    fn get_status(&self, path: &Path) -> SyncStatus {
        if !self.is_managed(path) { return SyncStatus::Unknown; }

        if path.exists() { SyncStatus::Synced } else { SyncStatus::Offline }
    }
}

impl CloudStatusProvider {
    pub fn new() -> Self {
        let providers: Vec<Box<dyn CloudProvider>> = vec![
            Box::new(NextcloudProvider::new()),
            Box::new(GDriveProvider::new()),
            Box::new(DropboxProvider::new()),
        ];
        Self {
            providers,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get sync status for a file
    pub fn get_status(&self, path: &Path) -> SyncStatus {
        // Check cache first
        if let Ok(cache) = self.cache.lock() {
            if let Some(status) = cache.get(path) {
                return *status;
            }
        }

        // Check each provider
        for provider in &self.providers {
            if provider.is_managed(path) {
                let status = provider.get_status(path);
                if let Ok(mut cache) = self.cache.lock() {
                    cache.insert(path.to_path_buf(), status);
                }
                return status;
            }
        }

        SyncStatus::Unknown
    }

    /// Check if any provider manages files in the given directory
    pub fn has_cloud_files(&self, dir: &Path) -> bool {
        self.providers.iter().any(|p| p.is_managed(dir))
    }

    /// Clear the status cache (call when directory changes)
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}

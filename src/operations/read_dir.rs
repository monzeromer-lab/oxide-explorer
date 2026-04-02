use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified: i64,
    pub is_dir: bool,
    pub icon_name: String,
    pub content_type: String,
}

pub async fn read_directory(path: &Path) -> Result<Vec<DirEntry>, std::io::Error> {
    let mut entries = Vec::new();
    let mut dir = tokio::fs::read_dir(path).await?;

    while let Some(entry) = dir.next_entry().await? {
        let metadata = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let icon_name = if is_dir {
            "folder".to_string()
        } else {
            guess_icon(&name)
        };

        let content_type = if is_dir {
            "inode/directory".to_string()
        } else {
            guess_content_type(&name)
        };

        entries.push(DirEntry {
            name,
            path: entry.path(),
            size,
            modified,
            is_dir,
            icon_name,
            content_type,
        });
    }

    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

fn guess_icon(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "java" | "go" | "rb" | "sh" => {
            "text-x-generic-symbolic".to_string()
        }
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" => {
            "image-x-generic-symbolic".to_string()
        }
        "mp4" | "mkv" | "avi" | "mov" | "webm" => "video-x-generic-symbolic".to_string(),
        "mp3" | "flac" | "ogg" | "wav" | "aac" => "audio-x-generic-symbolic".to_string(),
        "pdf" => "x-office-document-symbolic".to_string(),
        "zip" | "tar" | "gz" | "xz" | "7z" | "rar" => {
            "package-x-generic-symbolic".to_string()
        }
        "txt" | "md" | "log" | "toml" | "yaml" | "yml" | "json" | "xml" | "csv" => {
            "text-x-generic-symbolic".to_string()
        }
        _ => "text-x-generic-symbolic".to_string(),
    }
}

fn guess_content_type(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "txt" => "text/plain",
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
    .to_string()
}

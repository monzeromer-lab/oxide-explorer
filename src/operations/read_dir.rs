use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified: i64,
    pub is_dir: bool,
    pub is_symlink: bool,
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
        let is_symlink = entry
            .file_type()
            .await
            .map(|ft| ft.is_symlink())
            .unwrap_or(false);
        let size = if is_dir { 0 } else { metadata.len() };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let (content_type, icon_name) = detect_content_type(&name, is_dir);

        entries.push(DirEntry {
            name,
            path: entry.path(),
            size,
            modified,
            is_dir,
            is_symlink,
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

fn detect_content_type(name: &str, is_dir: bool) -> (String, String) {
    if is_dir {
        return ("inode/directory".to_string(), "folder".to_string());
    }

    // Use GIO content type detection for proper system theme icons
    let (content_type, _uncertain) = gio::content_type_guess(Some(name), &[]);
    let icon_name = gio::content_type_get_generic_icon_name(&content_type)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "text-x-generic".to_string());

    (content_type.to_string(), icon_name)
}

use std::path::Path;
use tokio::fs;

pub async fn create_directory(path: &Path) -> Result<(), std::io::Error> {
    fs::create_dir(path).await
}

pub async fn rename(from: &Path, to: &Path) -> Result<(), std::io::Error> {
    fs::rename(from, to).await
}

pub async fn copy_file(from: &Path, to: &Path) -> Result<(), std::io::Error> {
    if from.is_dir() {
        copy_dir_recursive(from, to).await
    } else {
        fs::copy(from, to).await?;
        Ok(())
    }
}

pub async fn move_file(from: &Path, to: &Path) -> Result<(), std::io::Error> {
    // Try rename first (same filesystem, instant)
    match fs::rename(from, to).await {
        Ok(()) => Ok(()),
        Err(_) => {
            // Cross-filesystem: copy then delete
            copy_file(from, to).await?;
            delete(from).await
        }
    }
}

pub async fn delete(path: &Path) -> Result<(), std::io::Error> {
    if path.is_dir() {
        fs::remove_dir_all(path).await
    } else {
        fs::remove_file(path).await
    }
}

async fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(to).await?;
    let mut dir = fs::read_dir(from).await?;
    while let Some(entry) = dir.next_entry().await? {
        let dest = to.join(entry.file_name());
        if entry.file_type().await?.is_dir() {
            Box::pin(copy_dir_recursive(&entry.path(), &dest)).await?;
        } else {
            fs::copy(entry.path(), &dest).await?;
        }
    }
    Ok(())
}

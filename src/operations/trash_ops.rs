use std::path::Path;

pub fn move_to_trash(path: &Path) -> Result<(), trash::Error> {
    trash::delete(path)
}

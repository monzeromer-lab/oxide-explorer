use gio::prelude::*;
use std::path::Path;

pub fn move_to_trash(path: &Path) -> Result<(), glib::Error> {
    let file = gio::File::for_path(path);
    file.trash(gio::Cancellable::NONE)?;
    Ok(())
}

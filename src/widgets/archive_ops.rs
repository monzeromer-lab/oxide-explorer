use adw::prelude::*;
use std::path::{Path, PathBuf};

use crate::utils::runtime;

/// Show extract dialog for an archive file
pub fn show_extract_dialog<F>(parent: &impl IsA<gtk::Window>, archive: &Path, on_done: F)
where
    F: Fn(PathBuf) + Clone + 'static,
{
    let archive = archive.to_path_buf();
    let default_dest = archive.parent().unwrap_or(Path::new("/")).to_path_buf();
    let archive_name = archive.file_stem().unwrap_or_default().to_string_lossy().into_owned();

    let dialog = adw::MessageDialog::new(
        Some(parent),
        Some("Extract Archive"),
        Some(&format!("Extract \"{}\" to:", archive.file_name().unwrap_or_default().to_string_lossy())),
    );
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("extract", "Extract");
    dialog.set_response_appearance("extract", adw::ResponseAppearance::Suggested);

    let entry = gtk::Entry::new();
    entry.set_text(&default_dest.join(&archive_name).to_string_lossy());
    dialog.set_extra_child(Some(&entry));

    let entry_ref = entry.clone();
    dialog.connect_response(None, move |dlg, response| {
        if response == "extract" {
            let dest = PathBuf::from(entry_ref.text().to_string());
            let archive = archive.clone();
            let on_done = on_done.clone();

            glib::spawn_future_local(async move {
                let dest2 = dest.clone();
                let result = runtime::spawn(async move {
                    extract_archive(&archive, &dest2)
                }).await;

                match result {
                    Ok(Ok(())) => on_done(dest),
                    Ok(Err(e)) => log::error!("Extract failed: {e}"),
                    Err(e) => log::error!("Task failed: {e}"),
                }
            });
        }
        dlg.close();
    });
    dialog.present();
}

/// Show compress dialog
pub fn show_compress_dialog<F>(parent: &impl IsA<gtk::Window>, files: &[PathBuf], on_done: F)
where
    F: Fn(PathBuf) + Clone + 'static,
{
    if files.is_empty() { return; }

    let default_name = if files.len() == 1 {
        let name = files[0].file_name().unwrap_or_default().to_string_lossy().into_owned();
        format!("{name}.tar.gz")
    } else {
        "archive.tar.gz".to_string()
    };
    let dest_dir = files[0].parent().unwrap_or(Path::new("/")).to_path_buf();

    let dialog = adw::MessageDialog::new(
        Some(parent),
        Some("Create Archive"),
        Some(&format!("Compress {} item(s):", files.len())),
    );
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("compress", "Create");
    dialog.set_response_appearance("compress", adw::ResponseAppearance::Suggested);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);

    let name_entry = gtk::Entry::new();
    name_entry.set_text(&default_name);
    name_entry.set_placeholder_text(Some("Archive name"));
    vbox.append(&name_entry);

    let format_row = adw::ComboRow::builder().title("Format").build();
    let formats = gtk::StringList::new(&["tar.gz", "zip", "tar.xz", "tar.bz2"]);
    format_row.set_model(Some(&formats));
    vbox.append(&format_row);

    dialog.set_extra_child(Some(&vbox));

    let files = files.to_vec();
    let name_ref = name_entry.clone();
    dialog.connect_response(None, move |dlg, response| {
        if response == "compress" {
            let archive_name = name_ref.text().to_string();
            let archive_path = dest_dir.join(&archive_name);
            let files = files.clone();
            let on_done = on_done.clone();

            glib::spawn_future_local(async move {
                let ap2 = archive_path.clone();
                let result = runtime::spawn(async move {
                    compress_files(&files, &ap2)
                }).await;

                match result {
                    Ok(Ok(())) => on_done(archive_path),
                    Ok(Err(e)) => log::error!("Compress failed: {e}"),
                    Err(e) => log::error!("Task failed: {e}"),
                }
            });
        }
        dlg.close();
    });
    dialog.present();
}

fn extract_archive(archive: &Path, dest: &Path) -> Result<(), String> {
    let _ = std::fs::create_dir_all(dest);
    let ext = archive.to_string_lossy().to_lowercase();

    if ext.ends_with(".zip") {
        run_cmd("unzip", &["-o", &archive.to_string_lossy(), "-d", &dest.to_string_lossy()])
    } else if ext.ends_with(".tar.gz") || ext.ends_with(".tgz") {
        run_cmd("tar", &["xzf", &archive.to_string_lossy(), "-C", &dest.to_string_lossy()])
    } else if ext.ends_with(".tar.xz") || ext.ends_with(".txz") {
        run_cmd("tar", &["xJf", &archive.to_string_lossy(), "-C", &dest.to_string_lossy()])
    } else if ext.ends_with(".tar.bz2") || ext.ends_with(".tbz2") {
        run_cmd("tar", &["xjf", &archive.to_string_lossy(), "-C", &dest.to_string_lossy()])
    } else if ext.ends_with(".tar") {
        run_cmd("tar", &["xf", &archive.to_string_lossy(), "-C", &dest.to_string_lossy()])
    } else if ext.ends_with(".7z") {
        run_cmd("7z", &["x", &archive.to_string_lossy(), &format!("-o{}", dest.to_string_lossy())])
    } else if ext.ends_with(".rar") {
        run_cmd("unrar", &["x", &archive.to_string_lossy(), &dest.to_string_lossy()])
    } else {
        Err(format!("Unsupported archive format: {ext}"))
    }
}

fn compress_files(files: &[PathBuf], archive: &Path) -> Result<(), String> {
    let ext = archive.to_string_lossy().to_lowercase();
    let file_args: Vec<String> = files.iter().map(|f| {
        f.file_name().unwrap_or_default().to_string_lossy().into_owned()
    }).collect();
    let parent = files[0].parent().unwrap_or(Path::new("/"));

    if ext.ends_with(".zip") {
        let mut args = vec!["-r".to_string(), archive.to_string_lossy().into_owned()];
        args.extend(file_args);
        run_cmd_in("zip", &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(), parent)
    } else if ext.ends_with(".tar.gz") || ext.ends_with(".tgz") {
        let mut args = vec!["czf".to_string(), archive.to_string_lossy().into_owned()];
        args.extend(file_args);
        run_cmd_in("tar", &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(), parent)
    } else if ext.ends_with(".tar.xz") {
        let mut args = vec!["cJf".to_string(), archive.to_string_lossy().into_owned()];
        args.extend(file_args);
        run_cmd_in("tar", &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(), parent)
    } else {
        let mut args = vec!["cf".to_string(), archive.to_string_lossy().into_owned()];
        args.extend(file_args);
        run_cmd_in("tar", &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(), parent)
    }
}

fn run_cmd(program: &str, args: &[&str]) -> Result<(), String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if output.status.success() { Ok(()) }
    else { Err(String::from_utf8_lossy(&output.stderr).into_owned()) }
}

fn run_cmd_in(program: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    let output = std::process::Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if output.status.success() { Ok(()) }
    else { Err(String::from_utf8_lossy(&output.stderr).into_owned()) }
}

pub fn is_archive(path: &Path) -> bool {
    let ext = path.to_string_lossy().to_lowercase();
    ext.ends_with(".zip") || ext.ends_with(".tar.gz") || ext.ends_with(".tgz") ||
    ext.ends_with(".tar.xz") || ext.ends_with(".txz") || ext.ends_with(".tar.bz2") ||
    ext.ends_with(".tar") || ext.ends_with(".7z") || ext.ends_with(".rar")
}

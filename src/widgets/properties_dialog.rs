use adw::prelude::*;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

use crate::utils::format;
use crate::utils::runtime;

pub fn show_properties(parent: &impl IsA<gtk::Window>, path: &Path) {
    let path = path.to_path_buf();
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let dialog = adw::Window::builder()
        .title(&format!("{name} — Properties"))
        .default_width(400)
        .default_height(500)
        .modal(true)
        .transient_for(parent)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    // Name + icon
    let name_group = adw::PreferencesGroup::new();
    let name_row = adw::ActionRow::builder()
        .title("Name")
        .subtitle(&name)
        .build();
    name_group.add(&name_row);

    // Detect content type for icon
    let (content_type, _) = gio::content_type_guess(Some(&name), &[]);
    let type_desc = gio::content_type_get_description(&content_type);
    let type_row = adw::ActionRow::builder()
        .title("Type")
        .subtitle(&type_desc.to_string())
        .build();
    name_group.add(&type_row);

    let path_row = adw::ActionRow::builder()
        .title("Location")
        .subtitle(&*path.parent().unwrap_or(&path).to_string_lossy())
        .build();
    name_group.add(&path_row);
    content.append(&name_group);

    // Size group
    let size_group = adw::PreferencesGroup::new();
    let size_row = adw::ActionRow::builder()
        .title("Size")
        .subtitle("Calculating...")
        .build();
    size_group.add(&size_row);
    content.append(&size_group);

    // Dates group
    let dates_group = adw::PreferencesGroup::builder()
        .title("Dates")
        .build();

    if let Ok(meta) = std::fs::metadata(&path) {
        // Size (immediate for files)
        if !meta.is_dir() {
            size_row.set_subtitle(&format!(
                "{} ({} bytes)",
                format::format_size(meta.len()),
                meta.len()
            ));
        }

        if let Ok(modified) = meta.modified() {
            let ts = modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let modified_row = adw::ActionRow::builder()
                .title("Modified")
                .subtitle(&format::format_date(ts))
                .build();
            dates_group.add(&modified_row);
        }

        if let Ok(accessed) = meta.accessed() {
            let ts = accessed
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let accessed_row = adw::ActionRow::builder()
                .title("Accessed")
                .subtitle(&format::format_date(ts))
                .build();
            dates_group.add(&accessed_row);
        }

        // Unix permissions
        #[cfg(unix)]
        {
            let perms_group = adw::PreferencesGroup::builder()
                .title("Permissions")
                .build();

            let mode = meta.mode();
            let perms_str = format_permissions(mode);
            let perms_row = adw::ActionRow::builder()
                .title("Access")
                .subtitle(&format!("{perms_str} ({:o})", mode & 0o7777))
                .build();
            perms_group.add(&perms_row);

            let uid = meta.uid();
            let gid = meta.gid();

            let owner_name = nix::unistd::User::from_uid(nix::unistd::Uid::from_raw(uid))
                .ok()
                .flatten()
                .map(|u| u.name)
                .unwrap_or_else(|| uid.to_string());
            let group_name = nix::unistd::Group::from_gid(nix::unistd::Gid::from_raw(gid))
                .ok()
                .flatten()
                .map(|g| g.name)
                .unwrap_or_else(|| gid.to_string());

            let owner_row = adw::ActionRow::builder()
                .title("Owner")
                .subtitle(&format!("{owner_name}:{group_name}"))
                .build();
            perms_group.add(&owner_row);

            content.append(&perms_group);
        }
    }

    content.append(&dates_group);
    scroll.set_child(Some(&content));
    toolbar_view.set_content(Some(&scroll));
    dialog.set_content(Some(&toolbar_view));

    // For directories, calculate size asynchronously
    let dir_path = path.clone();
    if path.is_dir() {
        let size_row_ref = size_row.clone();
        glib::spawn_future_local(async move {
            let result = runtime::spawn(async move {
                calculate_dir_size(&dir_path)
            })
            .await;

            match result {
                Ok(size) => {
                    size_row_ref.set_subtitle(&format!(
                        "{} ({} bytes)",
                        format::format_size(size),
                        size
                    ));
                }
                Err(_) => {
                    size_row_ref.set_subtitle("Unable to calculate");
                }
            }
        });
    }

    dialog.present();
}

fn calculate_dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() {
                    total += calculate_dir_size(&entry.path());
                } else {
                    total += meta.len();
                }
            }
        }
    }
    total
}

#[cfg(unix)]
fn format_permissions(mode: u32) -> String {
    let mut s = String::with_capacity(10);
    // File type
    s.push(if mode & 0o40000 != 0 {
        'd'
    } else if mode & 0o120000 == 0o120000 {
        'l'
    } else {
        '-'
    });
    // Owner
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    // Group
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    // Other
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    s
}

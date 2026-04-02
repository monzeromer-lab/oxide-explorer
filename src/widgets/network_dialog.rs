use adw::prelude::*;
use std::path::PathBuf;

/// Dialog for connecting to network locations (SMB, FTP, SFTP, WebDAV)
pub fn show_connect_dialog<F>(parent: &impl IsA<gtk::Window>, on_connect: F)
where
    F: Fn(String) + Clone + 'static,
{
    let dialog = adw::Window::builder()
        .title("Connect to Server")
        .default_width(450)
        .default_height(400)
        .modal(true)
        .transient_for(parent)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_start(24);
    content.set_margin_end(24);
    content.set_margin_top(16);
    content.set_margin_bottom(24);

    // Protocol selector
    let protocol_group = adw::PreferencesGroup::builder()
        .title("Connection Type")
        .build();

    let protocol_row = adw::ComboRow::builder()
        .title("Protocol")
        .build();
    let protocols = gtk::StringList::new(&["SSH/SFTP", "FTP", "SMB (Windows Share)", "WebDAV"]);
    protocol_row.set_model(Some(&protocols));
    protocol_group.add(&protocol_row);
    content.append(&protocol_group);

    // Server details
    let server_group = adw::PreferencesGroup::builder()
        .title("Server Details")
        .build();

    let server_row = adw::EntryRow::builder()
        .title("Server Address")
        .build();
    server_group.add(&server_row);

    let port_row = adw::EntryRow::builder()
        .title("Port (optional)")
        .build();
    server_group.add(&port_row);

    let path_row = adw::EntryRow::builder()
        .title("Path (optional)")
        .text("/")
        .build();
    server_group.add(&path_row);

    let user_row = adw::EntryRow::builder()
        .title("Username (optional)")
        .build();
    server_group.add(&user_row);

    content.append(&server_group);

    // Quick bookmarks
    let bookmarks_group = adw::PreferencesGroup::builder()
        .title("Recent Connections")
        .build();

    let recent = load_recent_connections();
    for uri in &recent {
        let row = adw::ActionRow::builder()
            .title(uri)
            .activatable(true)
            .build();
        row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
        let uri_clone = uri.clone();
        let on_connect_ref = on_connect.clone();
        let dlg = dialog.clone();
        row.connect_activated(move |_| {
            on_connect_ref(uri_clone.clone());
            dlg.close();
        });
        bookmarks_group.add(&row);
    }
    content.append(&bookmarks_group);

    // Connect button
    let connect_btn = gtk::Button::with_label("Connect");
    connect_btn.add_css_class("suggested-action");
    connect_btn.set_halign(gtk::Align::End);
    connect_btn.set_margin_top(8);

    let server = server_row.clone();
    let port = port_row.clone();
    let path = path_row.clone();
    let user = user_row.clone();
    let proto = protocol_row.clone();
    let dlg = dialog.clone();
    let on_connect = std::rc::Rc::new(on_connect);
    connect_btn.connect_clicked(move |_| {
        let server_text = server.text().to_string();
        if server_text.is_empty() {
            return;
        }

        let protocol = match proto.selected() {
            0 => "sftp",
            1 => "ftp",
            2 => "smb",
            3 => "dav",
            _ => "sftp",
        };

        let port_text = port.text().to_string();
        let path_text = path.text().to_string();
        let user_text = user.text().to_string();

        let mut uri = String::new();
        uri.push_str(protocol);
        uri.push_str("://");
        if !user_text.is_empty() {
            uri.push_str(&user_text);
            uri.push('@');
        }
        uri.push_str(&server_text);
        if !port_text.is_empty() {
            uri.push(':');
            uri.push_str(&port_text);
        }
        if !path_text.is_empty() && !path_text.starts_with('/') {
            uri.push('/');
        }
        uri.push_str(&path_text);

        save_recent_connection(&uri);
        on_connect(uri.clone());
        dlg.close();
    });
    content.append(&connect_btn);

    toolbar_view.set_content(Some(&content));
    dialog.set_content(Some(&toolbar_view));
    dialog.present();
}

/// Mount a GIO URI and navigate to it
pub fn mount_and_navigate<F>(uri: &str, on_mounted: F)
where
    F: Fn(PathBuf) + 'static,
{
    let file = gio::File::for_uri(uri);
    let file2 = file.clone();
    let mount_op = gtk::MountOperation::new(gtk::Window::NONE);

    file.mount_enclosing_volume(
        gio::MountMountFlags::NONE,
        Some(&mount_op),
        gio::Cancellable::NONE,
        move |result| {
            match result {
                Ok(()) | Err(_) => {
                    // Even if "already mounted", try to get path
                    if let Some(path) = file2.path() {
                        on_mounted(path);
                    } else {
                        // For GVfs mounts, path is under /run/user/.../gvfs/
                        let uri = file2.uri();
                        log::info!("Mounted network location: {uri}");
                        // Try to find the GVfs mount point
                        if let Some(mount) = file2.find_enclosing_mount(gio::Cancellable::NONE).ok() {
                            if let Some(root_path) = mount.root().path() {
                                on_mounted(root_path);
                                return;
                            }
                        }
                        // Fallback: open via GIO URI
                        log::warn!("Could not get local path for {uri}");
                    }
                }
            }
        },
    );
}

fn recent_connections_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("oxide-explorer")
        .join("recent_connections.txt")
}

fn load_recent_connections() -> Vec<String> {
    let path = recent_connections_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => contents.lines().filter(|l| !l.is_empty()).map(String::from).collect(),
        Err(_) => Vec::new(),
    }
}

fn save_recent_connection(uri: &str) {
    let path = recent_connections_path();
    let mut recent = load_recent_connections();
    recent.retain(|u| u != uri);
    recent.insert(0, uri.to_string());
    recent.truncate(10); // keep last 10
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, recent.join("\n"));
}

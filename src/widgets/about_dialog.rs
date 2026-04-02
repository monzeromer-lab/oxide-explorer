use adw::prelude::*;

pub fn show_about(parent: &impl IsA<gtk::Window>) {
    let about = adw::AboutWindow::builder()
        .transient_for(parent)
        .application_name("Oxide Explorer")
        .application_icon("system-file-manager")
        .version("0.4.0")
        .developer_name("Monzer Omer")
        .license_type(gtk::License::MitX11)
        .website("https://github.com/oxide-explorer/oxide-explorer")
        .issue_url("https://github.com/oxide-explorer/oxide-explorer/issues")
        .copyright("2024-2026 Monzer Omer")
        .build();

    about.set_comments("A blazing-fast, power-user-centric file manager built with Rust, GTK4, and libadwaita. Combines the visual elegance of GNOME Files with the advanced productivity features of Directory Opus and Total Commander.");

    about.set_developers(&[
        "Monzer Omer <monzer.a.omer@gmail.com>",
    ]);

    about.add_acknowledgement_section(Some("Powered By"), &[
        "Rust https://rust-lang.org",
        "GTK4 https://gtk.org",
        "libadwaita https://gnome.pages.gitlab.gnome.org/libadwaita/",
        "Tokio https://tokio.rs",
        "VTE https://wiki.gnome.org/Apps/Terminal/VTE",
        "mlua (Lua 5.4) https://github.com/mlua-rs/mlua",
    ]);

    about.set_release_notes("
        <p>Version 0.4.0 - All Phases Complete</p>
        <ul>
            <li>Tabbed browsing with independent per-tab state</li>
            <li>Dual-pane Commander-style mode (F3)</li>
            <li>Embedded VTE terminal (F4) synced to CWD</li>
            <li>Miller columns Finder-style view (F5)</li>
            <li>Advanced search with regex and content search</li>
            <li>Batch file renamer with live preview</li>
            <li>Preview pane for text and images (Space)</li>
            <li>Archive extract and compress</li>
            <li>File tags and color dots via xattr</li>
            <li>Lua plugin system with context menu integration</li>
            <li>Network drive support (SMB, FTP, SFTP, WebDAV)</li>
            <li>Cloud sync status detection</li>
            <li>Vim-style keybindings and custom shortcuts</li>
            <li>Drag and drop, bookmarks, mounted volumes</li>
            <li>35+ keyboard shortcuts</li>
        </ul>
    ");

    about.present();
}

use adw::prelude::*;

pub fn show_about(parent: &impl IsA<gtk::Window>) {
    let about = adw::AboutWindow::builder()
        .transient_for(parent)
        .application_name("Oxide Explorer")
        .application_icon("system-file-manager")
        .version("0.2.0")
        .developer_name("Oxide Explorer Contributors")
        .license_type(gtk::License::MitX11)
        .website("https://github.com/oxide-explorer")
        .issue_url("https://github.com/oxide-explorer/issues")
        .copyright("2024-2026 Oxide Explorer Contributors")
        .build();

    about.set_comments("A blazing-fast, power-user-centric file manager built with Rust, GTK4, and libadwaita. Designed to combine the visual elegance of GNOME Files with the advanced productivity features of Directory Opus and Total Commander.");

    about.set_developers(&[
        "Oxide Explorer Contributors",
    ]);

    about.add_acknowledgement_section(Some("Powered By"), &[
        "Rust https://rust-lang.org",
        "GTK4 https://gtk.org",
        "libadwaita https://gnome.pages.gitlab.gnome.org/libadwaita/",
        "Tokio https://tokio.rs",
    ]);

    // Release notes
    about.set_release_notes("
        <p>Phase 2 Release</p>
        <ul>
            <li>Tabbed browsing with independent per-tab state</li>
            <li>Dual-pane mode (F3) for Commander-style file management</li>
            <li>Embedded terminal panel (F4) synced to current directory</li>
            <li>Miller columns view (F5) for Finder-style navigation</li>
            <li>Vim-style keybindings (optional, enable in Preferences)</li>
            <li>Drag and drop file moving</li>
            <li>Bookmarks with persistence</li>
            <li>Mounted volume detection</li>
            <li>File properties dialog with permissions</li>
            <li>Preferences window with settings persistence</li>
            <li>Custom CSS styling</li>
            <li>Open With... dialog</li>
            <li>Undo trash with toast notifications</li>
            <li>30+ keyboard shortcuts</li>
        </ul>
    ");

    about.present();
}

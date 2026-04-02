use adw::prelude::*;

pub fn show_guide(parent: &impl IsA<gtk::Window>) {
    let window = adw::Window::builder()
        .title("Oxide Explorer Guide")
        .default_width(650)
        .default_height(700)
        .modal(true)
        .transient_for(parent)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
    content.set_margin_start(24);
    content.set_margin_end(24);
    content.set_margin_top(16);
    content.set_margin_bottom(24);

    // --- Welcome ---
    let welcome = adw::StatusPage::new();
    welcome.set_icon_name(Some("folder-open-symbolic"));
    welcome.set_title("Welcome to Oxide Explorer");
    welcome.set_description(Some("A blazing-fast, power-user file manager built with Rust and GTK4."));
    content.append(&welcome);

    // --- Getting Started ---
    add_section(&content, "Getting Started", &[
        ("Navigate", "Click folders to open them, or double-click files to launch them with the default app."),
        ("Breadcrumb Bar", "Click any segment in the path bar to jump to that directory. Press Ctrl+L to type a path directly."),
        ("Views", "Toggle between Icon View and Details View using the button in the header bar. Details View shows sortable columns."),
        ("Tabs", "Open new tabs with Ctrl+T. Close with Ctrl+W. Each tab has independent navigation and state."),
    ]);

    // --- Navigation shortcuts ---
    add_shortcut_section(&content, "Navigation", &[
        ("Alt+Left / Backspace", "Go back"),
        ("Alt+Right", "Go forward"),
        ("Alt+Up", "Parent directory"),
        ("Ctrl+L", "Type a path"),
        ("Enter", "Open selected item"),
    ]);

    // --- File operations ---
    add_shortcut_section(&content, "File Operations", &[
        ("Ctrl+C", "Copy selected files"),
        ("Ctrl+X", "Cut selected files"),
        ("Ctrl+V", "Paste"),
        ("Ctrl+Shift+C", "Copy file path to clipboard"),
        ("Delete", "Move to Trash (with confirmation)"),
        ("F2", "Rename"),
        ("Ctrl+A", "Select all"),
        ("Ctrl+I", "Invert selection"),
    ]);

    // --- View & Tools ---
    add_shortcut_section(&content, "View & Tools", &[
        ("Ctrl+H", "Toggle hidden files"),
        ("Ctrl+F", "Filter / search files"),
        ("Ctrl+D", "Bookmark current folder"),
        ("Ctrl++/-/0", "Zoom in / out / reset"),
        ("Alt+Enter", "File properties"),
        ("Ctrl+Alt+T", "Open external terminal"),
    ]);

    // --- Phase 2 features ---
    add_shortcut_section(&content, "Power User Features", &[
        ("F3", "Toggle Dual Pane mode (Commander-style split view)"),
        ("F4", "Toggle embedded Terminal panel"),
        ("F5", "Toggle Miller Columns view"),
        ("F6", "Copy selection to other pane (dual pane)"),
        ("Shift+F6", "Move selection to other pane (dual pane)"),
        ("Tab", "Switch focus between dual panes"),
        ("Ctrl+T", "New tab"),
        ("Ctrl+W", "Close tab"),
        ("Ctrl+,", "Preferences"),
    ]);

    // --- Vim mode ---
    add_section(&content, "Vim Keybindings", &[
        ("Enable", "Go to Preferences (Ctrl+,) and enable \"Vim-style Keybindings\" under Keybindings. Restart the app to apply."),
        ("Navigation", "h = parent, j = down, k = up, l = open, g = first, G = last"),
        ("Actions", "d = trash, y = copy, p = paste, x = cut, r = rename"),
        ("Tools", "/ = filter, . = toggle hidden, t = new tab, o = terminal, i = properties, ? = help"),
    ]);

    // --- Dual Pane ---
    add_section(&content, "Dual Pane Mode (F3)", &[
        ("Overview", "Split the view into two independent file browsers side by side, like Total Commander."),
        ("Focus", "Click on a pane to make it active. File operations act on the active pane."),
        ("Workflow", "Great for copying/moving files between two directories."),
    ]);

    // --- Miller Columns ---
    add_section(&content, "Miller Columns (F5)", &[
        ("Overview", "macOS Finder-style cascading columns for fast deep-tree navigation."),
        ("Usage", "Click a folder in any column to open its contents in the next column. Click a file to open it."),
        ("Navigation", "Scroll horizontally to see deeper levels of the directory tree."),
    ]);

    // --- Terminal ---
    add_section(&content, "Embedded Terminal (F4)", &[
        ("Overview", "A full terminal emulator embedded at the bottom of the window."),
        ("Shell", "Uses your $SHELL environment variable (defaults to /bin/bash)."),
        ("Sync", "Opening the terminal automatically runs 'cd' to your current directory."),
    ]);

    // --- Sidebar ---
    add_section(&content, "Sidebar", &[
        ("Places", "Quick access to Home, Documents, Downloads, Desktop, and other standard folders."),
        ("Devices", "Automatically detects mounted drives (USB, network, etc.)."),
        ("Bookmarks", "Press Ctrl+D to bookmark the current folder. Bookmarks appear in the sidebar."),
    ]);

    // --- Tips ---
    let tips_group = adw::PreferencesGroup::builder()
        .title("Tips")
        .build();

    let tips = [
        "Right-click anywhere for context menu with all available actions.",
        "Drag files from the icon view to move them to other directories.",
        "Use \"Open With...\" in the context menu to choose a specific app.",
        "Settings are saved to ~/.config/oxide-explorer/config.toml.",
        "The filter bar (Ctrl+F) hides non-matching files in real-time.",
    ];

    for tip in tips {
        let row = adw::ActionRow::builder()
            .title(tip)
            .build();
        row.add_prefix(&gtk::Image::from_icon_name("dialog-information-symbolic"));
        tips_group.add(&row);
    }
    content.append(&tips_group);

    scroll.set_child(Some(&content));
    toolbar_view.set_content(Some(&scroll));
    window.set_content(Some(&toolbar_view));
    window.present();
}

fn add_section(parent: &gtk::Box, title: &str, items: &[(&str, &str)]) {
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .build();
    for (label, desc) in items {
        let row = adw::ActionRow::builder()
            .title(*label)
            .subtitle(*desc)
            .build();
        group.add(&row);
    }
    parent.append(&group);
}

fn add_shortcut_section(parent: &gtk::Box, title: &str, shortcuts: &[(&str, &str)]) {
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .build();
    for (key, action) in shortcuts {
        let row = adw::ActionRow::builder()
            .title(*action)
            .build();
        let key_label = gtk::Label::new(Some(key));
        key_label.add_css_class("dim-label");
        key_label.set_width_request(160);
        key_label.set_halign(gtk::Align::End);
        row.add_suffix(&key_label);
        group.add(&row);
    }
    parent.append(&group);
}

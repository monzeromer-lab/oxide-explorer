use gio::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub struct Sidebar {
    pub widget: gtk::ScrolledWindow,
    pub list_box: gtk::ListBox,
    bookmarks: Rc<RefCell<Vec<PathBuf>>>,
    bookmarks_group: gtk::Box,
    on_navigate: Rc<dyn Fn(PathBuf)>,
}

struct PlaceItem {
    label: &'static str,
    icon: &'static str,
    path: PathBuf,
}

impl Sidebar {
    pub fn new<F>(on_navigate: F) -> Self
    where
        F: Fn(PathBuf) + Clone + 'static,
    {
        let on_navigate: Rc<dyn Fn(PathBuf)> = Rc::new(on_navigate.clone());

        let list_box = gtk::ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::Single);
        list_box.add_css_class("navigation-sidebar");

        let home = glib::home_dir();
        let places = vec![
            PlaceItem { label: "Home", icon: "user-home-symbolic", path: home.clone() },
            PlaceItem { label: "Documents", icon: "folder-documents-symbolic", path: home.join("Documents") },
            PlaceItem { label: "Downloads", icon: "folder-download-symbolic", path: home.join("Downloads") },
            PlaceItem { label: "Desktop", icon: "user-desktop-symbolic", path: home.join("Desktop") },
            PlaceItem { label: "Pictures", icon: "folder-pictures-symbolic", path: home.join("Pictures") },
            PlaceItem { label: "Music", icon: "folder-music-symbolic", path: home.join("Music") },
            PlaceItem { label: "Videos", icon: "folder-videos-symbolic", path: home.join("Videos") },
        ];

        // --- Places section ---
        let places_header = Self::create_header("Places");
        list_box.append(&places_header);

        for place in places {
            if !place.path.exists() { continue; }
            let row = Self::create_row(place.label, place.icon);
            row.set_activatable(true);
            row.set_widget_name(&place.path.to_string_lossy());
            list_box.append(&row);
        }

        // Filesystem root
        let root_row = Self::create_row("File System", "drive-harddisk-symbolic");
        root_row.set_activatable(true);
        root_row.set_widget_name("/");
        list_box.append(&root_row);

        // Trash entry
        let trash_row = Self::create_row("Trash", "user-trash-symbolic");
        trash_row.set_widget_name("trash:///");
        list_box.append(&trash_row);

        // --- Devices section ---
        let devices_header = Self::create_header("Devices");
        list_box.append(&devices_header);

        // Add mounted volumes via GIO VolumeMonitor
        let volume_monitor = gio::VolumeMonitor::get();
        let mounts = volume_monitor.mounts();

        for mount in &mounts {
            let name = mount.name();
            let root = mount.root();
            if let Some(path) = root.path() {
                let path_str = path.to_string_lossy().to_string();
                // Skip the root filesystem, we already have it
                if path_str == "/" { continue; }
                let icon = mount_icon_name(&mount);
                let row = Self::create_row(&name, &icon);
                row.set_activatable(true);
                row.set_widget_name(&path_str);
                list_box.append(&row);
            }
        }

        // Listen for mount/unmount events
        let list_box_ref = list_box.clone();
        volume_monitor.connect_mount_added(move |_, mount| {
            let name = mount.name();
            if let Some(path) = mount.root().path() {
                let path_str = path.to_string_lossy().to_string();
                if path_str == "/" { return; }
                let icon = mount_icon_name(&mount);
                let row = Self::create_row(&name, &icon);
                row.set_activatable(true);
                row.set_widget_name(&path_str);
                list_box_ref.append(&row);
            }
        });

        // --- Bookmarks section ---
        let bookmarks_header = Self::create_header("Bookmarks");
        list_box.append(&bookmarks_header);

        let bookmarks_group = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let bookmarks_row = gtk::ListBoxRow::new();
        bookmarks_row.set_child(Some(&bookmarks_group));
        bookmarks_row.set_activatable(false);
        bookmarks_row.set_selectable(false);
        list_box.append(&bookmarks_row);

        // Load saved bookmarks
        let bookmarks = Rc::new(RefCell::new(load_bookmarks()));

        // Populate bookmark rows
        {
            let bm = bookmarks.borrow();
            let nav_bm = on_navigate.clone();
            for path in bm.iter() {
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let row_btn = gtk::Button::new();
                row_btn.add_css_class("flat");
                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                hbox.set_margin_start(8);
                hbox.set_margin_end(8);
                hbox.set_margin_top(2);
                hbox.set_margin_bottom(2);
                hbox.append(&gtk::Image::from_icon_name("folder-symbolic"));
                let label = gtk::Label::new(Some(&name));
                label.set_halign(gtk::Align::Start);
                hbox.append(&label);
                row_btn.set_child(Some(&hbox));
                let p = path.clone();
                let nav = nav_bm.clone();
                row_btn.connect_clicked(move |_| { nav(p.clone()); });
                bookmarks_group.append(&row_btn);
            }
        }

        // Connect row activation
        let nav_for_activate = on_navigate.clone();
        list_box.connect_row_activated(move |_, row| {
            let name = row.widget_name();
            if !name.is_empty() && name != "trash:///" {
                nav_for_activate(PathBuf::from(name.as_str()));
            }
        });

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled.set_child(Some(&list_box));
        scrolled.set_width_request(200);

        Self {
            widget: scrolled,
            list_box,
            bookmarks,
            bookmarks_group,
            on_navigate,
        }
    }

    pub fn add_bookmark(&self, path: PathBuf) {
        let mut bm = self.bookmarks.borrow_mut();
        if bm.contains(&path) { return; }
        bm.push(path.clone());
        save_bookmarks(&bm);

        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let row_btn = gtk::Button::new();
        row_btn.add_css_class("flat");
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        hbox.set_margin_start(8);
        hbox.set_margin_end(8);
        hbox.set_margin_top(2);
        hbox.set_margin_bottom(2);
        hbox.append(&gtk::Image::from_icon_name("folder-symbolic"));
        let label = gtk::Label::new(Some(&name));
        label.set_halign(gtk::Align::Start);
        hbox.append(&label);
        row_btn.set_child(Some(&hbox));
        let nav = self.on_navigate.clone();
        row_btn.connect_clicked(move |_| { nav(path.clone()); });
        self.bookmarks_group.append(&row_btn);
    }

    pub fn remove_bookmark(&self, path: &PathBuf) {
        let mut bm = self.bookmarks.borrow_mut();
        bm.retain(|p| p != path);
        save_bookmarks(&bm);

        // Remove the visual row
        while let Some(child) = self.bookmarks_group.first_child() {
            self.bookmarks_group.remove(&child);
        }
        // Re-populate
        let nav = self.on_navigate.clone();
        for p in bm.iter() {
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            let row_btn = gtk::Button::new();
            row_btn.add_css_class("flat");
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            hbox.set_margin_start(8);
            hbox.set_margin_end(8);
            hbox.set_margin_top(2);
            hbox.set_margin_bottom(2);
            hbox.append(&gtk::Image::from_icon_name("folder-symbolic"));
            let label = gtk::Label::new(Some(&name));
            label.set_halign(gtk::Align::Start);
            hbox.append(&label);
            row_btn.set_child(Some(&hbox));
            let p = p.clone();
            let nav = nav.clone();
            row_btn.connect_clicked(move |_| { nav(p.clone()); });
            self.bookmarks_group.append(&row_btn);
        }
    }

    fn create_header(title: &str) -> gtk::Label {
        let header = gtk::Label::new(Some(title));
        header.set_halign(gtk::Align::Start);
        header.add_css_class("heading");
        header.set_margin_start(12);
        header.set_margin_top(12);
        header.set_margin_bottom(4);
        header
    }

    fn create_row(label: &str, icon_name: &str) -> gtk::ListBoxRow {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        hbox.set_margin_start(8);
        hbox.set_margin_end(8);
        hbox.set_margin_top(4);
        hbox.set_margin_bottom(4);

        let icon = gtk::Image::from_icon_name(icon_name);
        hbox.append(&icon);

        let lbl = gtk::Label::new(Some(label));
        lbl.set_halign(gtk::Align::Start);
        hbox.append(&lbl);

        let row = gtk::ListBoxRow::new();
        row.set_child(Some(&hbox));
        row
    }
}

fn mount_icon_name(mount: &gio::Mount) -> String {
    let icon = mount.icon();
    // Try to get a themed icon name
    if let Some(themed) = icon.downcast_ref::<gio::ThemedIcon>() {
        let names = themed.names();
        // Prefer -symbolic variant
        for name in &names {
            if name.ends_with("-symbolic") {
                return name.to_string();
            }
        }
        if let Some(first) = names.first() {
            return first.to_string();
        }
    }
    "drive-harddisk-symbolic".to_string()
}

fn bookmarks_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("oxide-explorer")
        .join("bookmarks.txt")
}

fn load_bookmarks() -> Vec<PathBuf> {
    let path = bookmarks_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => contents
            .lines()
            .filter(|l| !l.is_empty())
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn save_bookmarks(bookmarks: &[PathBuf]) {
    let path = bookmarks_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let contents: String = bookmarks
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("\n");
    let _ = std::fs::write(&path, contents);
}

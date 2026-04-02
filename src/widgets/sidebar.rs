use gtk::prelude::*;
use std::path::PathBuf;

pub struct Sidebar {
    pub widget: gtk::ScrolledWindow,
    pub list_box: gtk::ListBox,
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
        let list_box = gtk::ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::Single);
        list_box.add_css_class("navigation-sidebar");

        let home = glib::home_dir();
        let places = vec![
            PlaceItem {
                label: "Home",
                icon: "user-home-symbolic",
                path: home.clone(),
            },
            PlaceItem {
                label: "Documents",
                icon: "folder-documents-symbolic",
                path: home.join("Documents"),
            },
            PlaceItem {
                label: "Downloads",
                icon: "folder-download-symbolic",
                path: home.join("Downloads"),
            },
            PlaceItem {
                label: "Desktop",
                icon: "user-desktop-symbolic",
                path: home.join("Desktop"),
            },
            PlaceItem {
                label: "Pictures",
                icon: "folder-pictures-symbolic",
                path: home.join("Pictures"),
            },
            PlaceItem {
                label: "Music",
                icon: "folder-music-symbolic",
                path: home.join("Music"),
            },
            PlaceItem {
                label: "Videos",
                icon: "folder-videos-symbolic",
                path: home.join("Videos"),
            },
        ];

        // Header
        let header = gtk::Label::new(Some("Places"));
        header.set_halign(gtk::Align::Start);
        header.add_css_class("heading");
        header.set_margin_start(12);
        header.set_margin_top(8);
        header.set_margin_bottom(4);
        list_box.append(&header);

        for place in places {
            if !place.path.exists() {
                continue;
            }
            let row = Self::create_row(place.label, place.icon);
            row.set_activatable(true);
            list_box.append(&row);

            // We connect to the list_box row-activated instead (below)
            // Store path as widget name for retrieval
            row.set_widget_name(&place.path.to_string_lossy());
        }

        // Trash entry
        let trash_row = Self::create_row("Trash", "user-trash-symbolic");
        trash_row.set_widget_name("trash:///");
        list_box.append(&trash_row);

        // Connect row activation
        let nav = on_navigate.clone();
        list_box.connect_row_activated(move |_, row| {
            let name = row.widget_name();
            if !name.is_empty() && name != "trash:///" {
                nav(PathBuf::from(name.as_str()));
            }
        });

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled.set_child(Some(&list_box));
        scrolled.set_width_request(200);

        Self {
            widget: scrolled,
            list_box,
        }
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

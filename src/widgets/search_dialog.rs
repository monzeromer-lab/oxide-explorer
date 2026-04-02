use adw::prelude::*;
use std::path::{Path, PathBuf};

use crate::utils::runtime;

pub fn show_search_dialog<F>(parent: &impl IsA<gtk::Window>, search_dir: &Path, on_open: F)
where
    F: Fn(PathBuf) + Clone + 'static,
{
    let dialog = adw::Window::builder()
        .title("Search Files")
        .default_width(600)
        .default_height(500)
        .modal(true)
        .transient_for(parent)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(8);
    content.set_margin_bottom(8);

    // Search entry
    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search pattern (supports regex and wildcards)..."));
    content.append(&search_entry);

    // Options row
    let opts = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let regex_check = gtk::CheckButton::with_label("Regex");
    let case_check = gtk::CheckButton::with_label("Case sensitive");
    let content_check = gtk::CheckButton::with_label("Search file contents");
    opts.append(&regex_check);
    opts.append(&case_check);
    opts.append(&content_check);
    content.append(&opts);

    // Status
    let status_label = gtk::Label::new(Some("Type to search..."));
    status_label.set_halign(gtk::Align::Start);
    status_label.add_css_class("dim-label");
    content.append(&status_label);

    // Results list
    let results_model = gtk::StringList::new(&[]);
    let selection = gtk::SingleSelection::new(Some(results_model.clone()));

    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        hbox.set_margin_start(4);
        hbox.set_margin_top(2);
        hbox.set_margin_bottom(2);
        let icon = gtk::Image::from_icon_name("text-x-generic-symbolic");
        icon.set_pixel_size(16);
        hbox.append(&icon);
        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);
        label.set_ellipsize(gtk::pango::EllipsizeMode::Start);
        label.set_hexpand(true);
        hbox.append(&label);
        item.set_child(Some(&hbox));
    });
    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let string_obj = item.item().and_downcast::<gtk::StringObject>().unwrap();
        let path_str = string_obj.string();
        let hbox = item.child().and_downcast::<gtk::Box>().unwrap();
        let icon = hbox.first_child().and_downcast::<gtk::Image>().unwrap();
        let label = icon.next_sibling().and_downcast::<gtk::Label>().unwrap();
        label.set_text(&path_str);
        // Set icon based on whether it's a directory
        let p = std::path::Path::new(path_str.as_str());
        if p.is_dir() {
            icon.set_icon_name(Some("folder-symbolic"));
        }
    });

    let list_view = gtk::ListView::new(Some(selection.clone()), Some(factory));
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_child(Some(&list_view));
    scrolled.set_vexpand(true);
    content.append(&scrolled);

    // Double-click to open
    let on_open_ref = on_open.clone();
    let sel_ref = selection.clone();
    let dlg = dialog.clone();
    list_view.connect_activate(move |_, pos| {
        if let Some(item) = sel_ref.item(pos) {
            let string_obj = item.downcast::<gtk::StringObject>().unwrap();
            let path = PathBuf::from(string_obj.string().as_str());
            if path.is_dir() {
                on_open_ref(path);
            } else if let Some(parent) = path.parent() {
                on_open_ref(parent.to_path_buf());
            }
            dlg.close();
        }
    });

    // Wire search
    let search_dir = search_dir.to_path_buf();
    let results = results_model.clone();
    let status = status_label.clone();
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_string();
        if query.len() < 2 {
            status.set_text("Type at least 2 characters...");
            return;
        }

        let is_regex = regex_check.is_active();
        let case_sensitive = case_check.is_active();
        let search_contents = content_check.is_active();
        let dir = search_dir.clone();
        let results = results.clone();
        let status = status.clone();

        status.set_text("Searching...");

        glib::spawn_future_local(async move {
            let result = runtime::spawn(async move {
                search_files(&dir, &query, is_regex, case_sensitive, search_contents)
            }).await;

            match result {
                Ok(matches) => {
                    let count = matches.len();
                    // Clear and repopulate
                    let n = results.n_items();
                    if n > 0 { results.splice(0, n, &[] as &[&str]); }
                    for m in &matches {
                        results.append(&m.to_string_lossy());
                    }
                    status.set_text(&format!("{count} result(s) found"));
                }
                Err(e) => {
                    status.set_text(&format!("Error: {e}"));
                }
            }
        });
    });

    toolbar_view.set_content(Some(&content));
    dialog.set_content(Some(&toolbar_view));
    dialog.present();
}

fn search_files(
    dir: &Path,
    query: &str,
    is_regex: bool,
    case_sensitive: bool,
    search_contents: bool,
) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let max_results = 500;

    let pattern = if is_regex {
        if case_sensitive {
            regex::Regex::new(query).ok()
        } else {
            regex::Regex::new(&format!("(?i){query}")).ok()
        }
    } else {
        // Convert wildcard to regex
        let escaped = regex::escape(query);
        let wildcard = escaped.replace(r"\*", ".*").replace(r"\?", ".");
        if case_sensitive {
            regex::Regex::new(&wildcard).ok()
        } else {
            regex::Regex::new(&format!("(?i){wildcard}")).ok()
        }
    };

    let Some(re) = pattern else { return results };

    for entry in walkdir::WalkDir::new(dir)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if results.len() >= max_results { break; }

        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();

        // Match filename
        if re.is_match(&name) {
            results.push(path.to_path_buf());
            continue;
        }

        // Optionally search file contents
        if search_contents && path.is_file() {
            if let Ok(contents) = std::fs::read_to_string(path) {
                if re.is_match(&contents) {
                    results.push(path.to_path_buf());
                }
            }
        }
    }

    results
}

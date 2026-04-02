use adw::prelude::*;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use crate::models::file_entry::FileEntry;
use crate::models::file_list_model::FileListModel;
use crate::operations::read_dir;
use crate::utils::runtime;

/// A single column in the Miller columns view
struct MillerColumn {
    widget: gtk::ScrolledWindow,
    list_view: gtk::ListView,
    model: FileListModel,
    selection: gtk::SingleSelection,
    path: PathBuf,
}

impl MillerColumn {
    fn new(path: PathBuf) -> Self {
        let model = FileListModel::new();
        let selection = gtk::SingleSelection::new(Some(model.clone()));
        selection.set_autoselect(false);

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            hbox.set_margin_start(6);
            hbox.set_margin_end(6);
            hbox.set_margin_top(2);
            hbox.set_margin_bottom(2);
            let icon = gtk::Image::new();
            icon.set_pixel_size(16);
            hbox.append(&icon);
            let label = gtk::Label::new(None);
            label.set_halign(gtk::Align::Start);
            label.set_hexpand(true);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            hbox.append(&label);
            // Arrow indicator for directories
            let arrow = gtk::Image::from_icon_name("go-next-symbolic");
            arrow.set_pixel_size(12);
            arrow.set_visible(false);
            hbox.append(&arrow);
            item.set_child(Some(&hbox));
        });

        factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let entry = item.item().and_downcast::<FileEntry>().unwrap();
            let hbox = item.child().and_downcast::<gtk::Box>().unwrap();

            let icon = hbox.first_child().and_downcast::<gtk::Image>().unwrap();
            icon.set_icon_name(Some(&entry.icon_name()));

            let label = icon.next_sibling().and_downcast::<gtk::Label>().unwrap();
            label.set_text(&entry.name());

            let arrow = label.next_sibling().and_downcast::<gtk::Image>().unwrap();
            arrow.set_visible(entry.is_dir());
        });

        let list_view = gtk::ListView::new(Some(selection.clone()), Some(factory));

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled.set_child(Some(&list_view));
        scrolled.set_width_request(250);
        scrolled.set_vexpand(true);

        Self { widget: scrolled, list_view, model, selection, path }
    }

    fn load(&self) {
        let model = self.model.clone();
        let path = self.path.clone();
        glib::spawn_future_local(async move {
            let result = runtime::spawn(async move {
                read_dir::read_directory(&path).await
            }).await;
            if let Ok(Ok(entries)) = result {
                let file_entries: Vec<FileEntry> = entries
                    .into_iter()
                    .map(|e| FileEntry::new(&e.name, &e.path.to_string_lossy(), e.size, e.modified, e.is_dir, e.is_symlink, &e.icon_name, &e.content_type))
                    .collect();
                model.replace_all(file_entries);
            }
        });
    }
}

pub struct MillerColumnsView {
    pub widget: gtk::ScrolledWindow,
    columns_box: gtk::Box,
    columns: Rc<RefCell<Vec<MillerColumn>>>,
    on_file_open: Rc<dyn Fn(PathBuf)>,
}

impl MillerColumnsView {
    pub fn new<F>(on_file_open: F) -> Self
    where
        F: Fn(PathBuf) + 'static,
    {
        let columns_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
        scrolled.set_child(Some(&columns_box));
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        let columns: Rc<RefCell<Vec<MillerColumn>>> = Rc::new(RefCell::new(Vec::new()));
        let on_file_open = Rc::new(on_file_open);

        Self { widget: scrolled, columns_box, columns, on_file_open }
    }

    pub fn navigate_to(&self, path: PathBuf) {
        // Clear existing columns
        {
            let mut cols = self.columns.borrow_mut();
            cols.clear();
        }
        while let Some(child) = self.columns_box.first_child() {
            self.columns_box.remove(&child);
        }

        // Build columns for each path component
        let mut current = PathBuf::new();
        let components: Vec<_> = path.components().collect();

        for (i, component) in components.iter().enumerate() {
            current.push(component);
            if !current.is_dir() { continue; }
            self.add_column(current.clone(), i);
        }
    }

    fn add_column(&self, path: PathBuf, depth: usize) {
        let col = MillerColumn::new(path.clone());
        col.load();

        // Add separator between columns
        if depth > 0 {
            let sep = gtk::Separator::new(gtk::Orientation::Vertical);
            self.columns_box.append(&sep);
        }
        self.columns_box.append(&col.widget);

        // Wire selection to open next column
        let columns_box = self.columns_box.clone();
        let columns = self.columns.clone();
        let on_open = self.on_file_open.clone();
        let this_depth = depth;

        col.selection.connect_selection_changed(move |sel, _, _| {
            let pos = sel.selected();
            if pos == gtk::INVALID_LIST_POSITION { return; }
            if let Some(item) = sel.item(pos) {
                if let Some(entry) = item.downcast_ref::<FileEntry>() {
                    let entry_path = PathBuf::from(entry.path());
                    if entry.is_dir() {
                        // Remove columns after this one
                        let mut cols = columns.borrow_mut();
                        while cols.len() > this_depth + 1 {
                            cols.pop();
                            // Remove last two children (separator + column)
                            if let Some(child) = columns_box.last_child() {
                                columns_box.remove(&child);
                            }
                            if let Some(child) = columns_box.last_child() {
                                columns_box.remove(&child);
                            }
                        }

                        // Add new column
                        let new_col = MillerColumn::new(entry_path);
                        new_col.load();
                        let sep = gtk::Separator::new(gtk::Orientation::Vertical);
                        columns_box.append(&sep);
                        columns_box.append(&new_col.widget);
                        cols.push(new_col);
                    } else {
                        // Open file
                        (on_open)(entry_path);
                    }
                }
            }
        });

        self.columns.borrow_mut().push(col);
    }
}

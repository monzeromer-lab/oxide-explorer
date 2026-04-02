use gtk::prelude::*;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct BreadcrumbBar {
    pub widget: gtk::Stack,
    buttons_box: gtk::Box,
    path_entry: gtk::Entry,
    editing: Rc<RefCell<bool>>,
}

impl BreadcrumbBar {
    pub fn new<F>(on_navigate: F) -> Self
    where
        F: Fn(PathBuf) + Clone + 'static,
    {
        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        stack.set_transition_duration(100);
        stack.set_hexpand(true);

        // Breadcrumb buttons mode
        let buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        buttons_box.add_css_class("linked");
        stack.add_named(&buttons_box, Some("breadcrumb"));

        // Editable path entry mode
        let path_entry = gtk::Entry::new();
        path_entry.set_hexpand(true);
        path_entry.set_placeholder_text(Some("Enter path..."));
        stack.add_named(&path_entry, Some("entry"));

        stack.set_visible_child_name("breadcrumb");

        let editing = Rc::new(RefCell::new(false));

        // Enter key in entry → navigate
        let nav = on_navigate.clone();
        let stack_ref = stack.clone();
        let editing_ref2 = editing.clone();
        path_entry.connect_activate(move |entry| {
            let text = entry.text().to_string();
            let path = PathBuf::from(&text);
            if path.is_absolute() {
                *editing_ref2.borrow_mut() = false;
                stack_ref.set_visible_child_name("breadcrumb");
                nav(path);
            }
        });

        // Escape key → cancel edit
        let stack_ref = stack.clone();
        let editing_ref3 = editing.clone();
        let key_controller = gtk::EventControllerKey::new();
        let stack_ref2 = stack_ref.clone();
        let editing_ref4 = editing_ref3.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                *editing_ref4.borrow_mut() = false;
                stack_ref2.set_visible_child_name("breadcrumb");
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        path_entry.add_controller(key_controller);

        let bar = Self {
            widget: stack,
            buttons_box,
            path_entry,
            editing,
        };
        bar.set_path(Path::new("/"), on_navigate);
        bar
    }

    pub fn set_path<F>(&self, path: &Path, on_navigate: F)
    where
        F: Fn(PathBuf) + Clone + 'static,
    {
        // Update entry text
        self.path_entry.set_text(&path.to_string_lossy());

        // Remove old children
        while let Some(child) = self.buttons_box.first_child() {
            self.buttons_box.remove(&child);
        }

        // Root button
        let root_btn = gtk::Button::with_label("/");
        root_btn.add_css_class("flat");
        let nav = on_navigate.clone();
        root_btn.connect_clicked(move |_| nav(PathBuf::from("/")));
        self.buttons_box.append(&root_btn);

        // Build path components
        let mut accumulated = PathBuf::from("/");
        for component in path.components() {
            use std::path::Component;
            match component {
                Component::RootDir => continue,
                Component::Normal(name) => {
                    accumulated.push(name);

                    // Separator
                    let sep = gtk::Label::new(Some("/"));
                    sep.add_css_class("dim-label");
                    self.buttons_box.append(&sep);

                    let label = name.to_string_lossy().to_string();
                    let btn = gtk::Button::with_label(&label);
                    btn.add_css_class("flat");
                    let target = accumulated.clone();
                    let nav = on_navigate.clone();
                    btn.connect_clicked(move |_| nav(target.clone()));
                    self.buttons_box.append(&btn);
                }
                _ => {}
            }
        }
    }

    pub fn enter_edit_mode(&self) {
        *self.editing.borrow_mut() = true;
        self.widget.set_visible_child_name("entry");
        self.path_entry.grab_focus();
        self.path_entry.select_region(0, -1);
    }

}

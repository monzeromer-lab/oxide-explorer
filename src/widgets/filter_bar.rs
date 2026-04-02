use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct FilterBar {
    pub widget: gtk::Revealer,
    pub entry: gtk::SearchEntry,
    filter_text: Rc<RefCell<String>>,
}

impl FilterBar {
    pub fn new() -> Self {
        let entry = gtk::SearchEntry::new();
        entry.set_placeholder_text(Some("Type to filter..."));
        entry.set_hexpand(true);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        hbox.set_margin_start(8);
        hbox.set_margin_end(8);
        hbox.set_margin_top(4);
        hbox.set_margin_bottom(4);

        let icon = gtk::Image::from_icon_name("edit-find-symbolic");
        hbox.append(&icon);
        hbox.append(&entry);

        let close_btn = gtk::Button::from_icon_name("window-close-symbolic");
        close_btn.add_css_class("flat");
        hbox.append(&close_btn);

        let revealer = gtk::Revealer::new();
        revealer.set_child(Some(&hbox));
        revealer.set_reveal_child(false);
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);

        let filter_text = Rc::new(RefCell::new(String::new()));

        // Close button hides the filter bar
        let revealer_ref = revealer.clone();
        let entry_ref = entry.clone();
        let ft = filter_text.clone();
        close_btn.connect_clicked(move |_| {
            revealer_ref.set_reveal_child(false);
            entry_ref.set_text("");
            *ft.borrow_mut() = String::new();
        });

        // Escape key hides filter bar
        let revealer_ref = revealer.clone();
        let entry_ref2 = entry.clone();
        let ft = filter_text.clone();
        entry.connect_stop_search(move |_| {
            revealer_ref.set_reveal_child(false);
            entry_ref2.set_text("");
            *ft.borrow_mut() = String::new();
        });

        Self {
            widget: revealer,
            entry,
            filter_text,
        }
    }

    pub fn toggle(&self) {
        let visible = self.widget.reveals_child();
        self.widget.set_reveal_child(!visible);
        if !visible {
            self.entry.grab_focus();
        } else {
            self.entry.set_text("");
            *self.filter_text.borrow_mut() = String::new();
        }
    }

    pub fn show(&self) {
        self.widget.set_reveal_child(true);
        self.entry.grab_focus();
    }

    pub fn hide(&self) {
        self.widget.set_reveal_child(false);
        self.entry.set_text("");
        *self.filter_text.borrow_mut() = String::new();
    }

    pub fn is_visible(&self) -> bool {
        self.widget.reveals_child()
    }

    pub fn filter_text(&self) -> String {
        self.filter_text.borrow().clone()
    }
}

use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use super::details_view::DetailsView;
use super::icon_view::IconView;

pub struct ContentView {
    pub widget: gtk::Stack,
    pub icon_view: IconView,
    pub details_view: DetailsView,
    pub empty_view: adw::StatusPage,
    pub loading_spinner: gtk::Spinner,
    pub outer_stack: gtk::Stack,
}

impl ContentView {
    pub fn new(model: &impl IsA<gtk::SelectionModel>, icon_size: Rc<Cell<i32>>) -> Self {
        // Inner stack: icon vs details
        let view_stack = gtk::Stack::new();
        view_stack.set_vexpand(true);
        view_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        view_stack.set_transition_duration(150);

        let icon_view = IconView::new(model, icon_size);
        view_stack.add_named(&icon_view.widget, Some("icon"));

        let details_view = DetailsView::new(model);
        view_stack.add_named(&details_view.widget, Some("details"));

        view_stack.set_visible_child_name("icon");

        // Empty state
        let empty_view = adw::StatusPage::new();
        empty_view.set_icon_name(Some("folder-symbolic"));
        empty_view.set_title("Folder is Empty");
        empty_view.set_description(Some("This directory contains no files."));

        // Loading spinner
        let loading_spinner = gtk::Spinner::new();
        loading_spinner.set_halign(gtk::Align::Center);
        loading_spinner.set_valign(gtk::Align::Center);
        loading_spinner.set_width_request(48);
        loading_spinner.set_height_request(48);

        let loading_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        loading_box.set_valign(gtk::Align::Center);
        loading_box.set_halign(gtk::Align::Center);
        loading_box.append(&loading_spinner);
        let loading_label = gtk::Label::new(Some("Loading..."));
        loading_label.add_css_class("dim-label");
        loading_box.append(&loading_label);

        // Outer stack: loading / content / empty
        let outer_stack = gtk::Stack::new();
        outer_stack.set_vexpand(true);
        outer_stack.set_hexpand(true);
        outer_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        outer_stack.set_transition_duration(100);
        outer_stack.add_named(&loading_box, Some("loading"));
        outer_stack.add_named(&view_stack, Some("content"));
        outer_stack.add_named(&empty_view, Some("empty"));
        outer_stack.set_visible_child_name("content");

        Self {
            widget: view_stack,
            icon_view,
            details_view,
            empty_view,
            loading_spinner,
            outer_stack,
        }
    }

    pub fn toggle_view(&self) -> bool {
        let current = self.widget.visible_child_name();
        let is_details = current.as_deref() == Some("icon");
        if is_details {
            self.widget.set_visible_child_name("details");
        } else {
            self.widget.set_visible_child_name("icon");
        }
        is_details
    }

    pub fn show_loading(&self) {
        self.loading_spinner.set_spinning(true);
        self.outer_stack.set_visible_child_name("loading");
    }

    pub fn show_content(&self, item_count: u32) {
        self.loading_spinner.set_spinning(false);
        if item_count == 0 {
            self.empty_view.set_icon_name(Some("folder-symbolic"));
            self.empty_view.set_title("Folder is Empty");
            self.empty_view
                .set_description(Some("This directory contains no files."));
            self.outer_stack.set_visible_child_name("empty");
        } else {
            self.outer_stack.set_visible_child_name("content");
        }
    }

    pub fn show_error(&self, message: &str) {
        self.loading_spinner.set_spinning(false);
        self.empty_view
            .set_icon_name(Some("dialog-error-symbolic"));
        self.empty_view.set_title("Cannot Open Folder");
        self.empty_view.set_description(Some(message));
        self.outer_stack.set_visible_child_name("empty");
    }
}

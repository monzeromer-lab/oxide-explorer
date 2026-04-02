use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::models::file_entry::FileEntry;

pub struct IconView {
    pub widget: gtk::ScrolledWindow,
    pub grid_view: gtk::GridView,
    pub icon_size: Rc<Cell<i32>>,
}

impl IconView {
    pub fn new(model: &impl IsA<gtk::SelectionModel>, icon_size: Rc<Cell<i32>>) -> Self {
        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
            container.set_halign(gtk::Align::Center);
            container.set_margin_top(8);
            container.set_margin_bottom(8);

            let icon = gtk::Image::new();
            container.append(&icon);

            let label = gtk::Label::new(None);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_max_width_chars(14);
            label.set_lines(2);
            label.set_wrap(true);
            label.set_justify(gtk::Justification::Center);
            container.append(&label);

            // Symlink overlay indicator
            let overlay_label = gtk::Label::new(None);
            overlay_label.set_visible(false);
            overlay_label.add_css_class("dim-label");
            overlay_label.set_halign(gtk::Align::Start);
            container.append(&overlay_label);

            item.set_child(Some(&container));
        });

        let size = icon_size.clone();
        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let entry = item.item().and_downcast::<FileEntry>().unwrap();
            let container = item.child().and_downcast::<gtk::Box>().unwrap();

            let icon = container.first_child().and_downcast::<gtk::Image>().unwrap();
            icon.set_icon_name(Some(&entry.icon_name()));
            icon.set_pixel_size(size.get());
            container.set_width_request(size.get() + 48);

            let label = icon.next_sibling().and_downcast::<gtk::Label>().unwrap();
            label.set_text(&entry.name());

            // Symlink indicator
            let overlay = label.next_sibling().and_downcast::<gtk::Label>().unwrap();
            if entry.is_symlink() {
                overlay.set_text("↗ link");
                overlay.set_visible(true);
            } else {
                overlay.set_visible(false);
            }
        });

        let grid_view = gtk::GridView::new(Some(model.clone().upcast()), Some(factory));
        grid_view.set_min_columns(3);
        grid_view.set_max_columns(20);

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);
        scrolled.set_child(Some(&grid_view));

        Self {
            widget: scrolled,
            grid_view,
            icon_size,
        }
    }

    pub fn refresh_icons(&self) {
        // Force re-bind by re-setting the factory isn't great,
        // but we can trigger items_changed on the model if needed.
        // For now, changing icon size takes effect on next directory load.
    }
}

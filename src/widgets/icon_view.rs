use gtk::prelude::*;

use crate::models::file_entry::FileEntry;

pub struct IconView {
    pub widget: gtk::ScrolledWindow,
    pub grid_view: gtk::GridView,
}

impl IconView {
    pub fn new(model: &impl IsA<gtk::SelectionModel>) -> Self {
        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
            container.set_halign(gtk::Align::Center);
            container.set_margin_top(8);
            container.set_margin_bottom(8);
            container.set_width_request(96);

            let icon = gtk::Image::new();
            icon.set_pixel_size(48);
            container.append(&icon);

            let label = gtk::Label::new(None);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_max_width_chars(14);
            label.set_lines(2);
            label.set_wrap(true);
            label.set_justify(gtk::Justification::Center);
            container.append(&label);

            item.set_child(Some(&container));
        });

        factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let entry = item.item().and_downcast::<FileEntry>().unwrap();
            let container = item.child().and_downcast::<gtk::Box>().unwrap();

            let icon = container.first_child().and_downcast::<gtk::Image>().unwrap();
            icon.set_icon_name(Some(&entry.icon_name()));

            let label = icon
                .next_sibling()
                .and_downcast::<gtk::Label>()
                .unwrap();
            label.set_text(&entry.name());
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
        }
    }
}

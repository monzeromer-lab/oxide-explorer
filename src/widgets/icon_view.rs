use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::models::file_entry::FileEntry;

pub struct IconView {
    pub widget: gtk::ScrolledWindow,
    pub grid_view: gtk::GridView,
}

impl IconView {
    pub fn new(model: &impl IsA<gtk::SelectionModel>, icon_size: Rc<Cell<i32>>) -> Self {
        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let container = gtk::Box::new(gtk::Orientation::Vertical, 2);
            container.set_halign(gtk::Align::Center);
            container.set_valign(gtk::Align::Start);
            container.set_margin_top(4);
            container.set_margin_bottom(4);
            container.set_margin_start(4);
            container.set_margin_end(4);

            let icon = gtk::Image::new();
            container.append(&icon);

            let label = gtk::Label::new(None);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_width_chars(1);    // minimum allocation
            label.set_max_width_chars(14); // ellipsize after this
            label.set_lines(2);
            label.set_wrap(true);
            label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
            label.set_justify(gtk::Justification::Center);
            label.add_css_class("caption");
            container.append(&label);

            item.set_child(Some(&container));
        });

        let size = icon_size.clone();
        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let entry = item.item().and_downcast::<FileEntry>().unwrap();
            let container = item.child().and_downcast::<gtk::Box>().unwrap();

            let s = size.get();
            container.set_width_request(s + 24);

            let icon = container.first_child().and_downcast::<gtk::Image>().unwrap();
            icon.set_icon_name(Some(&entry.icon_name()));
            icon.set_pixel_size(s);

            let label = icon.next_sibling().and_downcast::<gtk::Label>().unwrap();
            label.set_text(&entry.name());
        });

        let grid_view = gtk::GridView::new(Some(model.clone().upcast()), Some(factory));
        grid_view.set_min_columns(4);
        grid_view.set_max_columns(30);

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

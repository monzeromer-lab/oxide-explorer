use gtk::prelude::*;

use crate::models::file_entry::FileEntry;
use crate::models::sort_model;
use crate::utils::format;

pub struct DetailsView {
    pub widget: gtk::ScrolledWindow,
    pub column_view: gtk::ColumnView,
}

impl DetailsView {
    pub fn new(model: &impl IsA<gtk::SelectionModel>) -> Self {
        let column_view = gtk::ColumnView::new(Some(model.clone().upcast()));
        column_view.set_show_column_separators(true);
        column_view.set_show_row_separators(true);

        // Name column
        let name_factory = gtk::SignalListItemFactory::new();
        name_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            hbox.set_margin_start(4);
            let icon = gtk::Image::new();
            icon.set_pixel_size(16);
            hbox.append(&icon);
            let label = gtk::Label::new(None);
            label.set_halign(gtk::Align::Start);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            hbox.append(&label);
            item.set_child(Some(&hbox));
        });
        name_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let entry = item.item().and_downcast::<FileEntry>().unwrap();
            let hbox = item.child().and_downcast::<gtk::Box>().unwrap();
            let icon = hbox.first_child().and_downcast::<gtk::Image>().unwrap();
            icon.set_icon_name(Some(&entry.icon_name()));
            let label = icon.next_sibling().and_downcast::<gtk::Label>().unwrap();
            label.set_text(&entry.name());
        });
        let name_col = gtk::ColumnViewColumn::new(Some("Name"), Some(name_factory));
        name_col.set_expand(true);
        name_col.set_sorter(Some(&sort_model::name_sorter()));
        column_view.append_column(&name_col);

        // Size column
        let size_factory = gtk::SignalListItemFactory::new();
        size_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::new(None);
            label.set_halign(gtk::Align::End);
            label.set_margin_end(8);
            item.set_child(Some(&label));
        });
        size_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let entry = item.item().and_downcast::<FileEntry>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            if entry.is_dir() {
                label.set_text("—");
            } else {
                label.set_text(&format::format_size(entry.size()));
            }
        });
        let size_col = gtk::ColumnViewColumn::new(Some("Size"), Some(size_factory));
        size_col.set_fixed_width(100);
        size_col.set_sorter(Some(&sort_model::size_sorter()));
        column_view.append_column(&size_col);

        // Date column
        let date_factory = gtk::SignalListItemFactory::new();
        date_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::new(None);
            label.set_halign(gtk::Align::Start);
            item.set_child(Some(&label));
        });
        date_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let entry = item.item().and_downcast::<FileEntry>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            label.set_text(&format::format_date(entry.modified()));
        });
        let date_col = gtk::ColumnViewColumn::new(Some("Modified"), Some(date_factory));
        date_col.set_fixed_width(160);
        date_col.set_sorter(Some(&sort_model::date_sorter()));
        column_view.append_column(&date_col);

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);
        scrolled.set_child(Some(&column_view));

        Self {
            widget: scrolled,
            column_view,
        }
    }
}

use gtk::prelude::*;

use crate::utils::format;

pub struct StatusBar {
    pub widget: gtk::Box,
    item_count_label: gtk::Label,
    selection_label: gtk::Label,
    disk_label: gtk::Label,
}

impl StatusBar {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        widget.set_margin_start(8);
        widget.set_margin_end(8);
        widget.set_margin_top(4);
        widget.set_margin_bottom(4);

        let item_count_label = gtk::Label::new(Some("0 items"));
        item_count_label.set_halign(gtk::Align::Start);
        item_count_label.add_css_class("dim-label");
        widget.append(&item_count_label);

        let selection_label = gtk::Label::new(None);
        selection_label.set_halign(gtk::Align::Center);
        selection_label.set_hexpand(true);
        selection_label.add_css_class("dim-label");
        widget.append(&selection_label);

        let disk_label = gtk::Label::new(None);
        disk_label.set_halign(gtk::Align::End);
        disk_label.add_css_class("dim-label");
        widget.append(&disk_label);

        Self {
            widget,
            item_count_label,
            selection_label,
            disk_label,
        }
    }

    pub fn set_item_count(&self, visible: u32, total: u32) {
        let text = if visible == total {
            if total == 1 {
                "1 item".to_string()
            } else {
                format!("{total} items")
            }
        } else {
            format!("{visible} of {total} items")
        };
        self.item_count_label.set_text(&text);
    }

    pub fn set_selection_info(&self, count: u32) {
        if count == 0 {
            self.selection_label.set_text("");
        } else {
            self.selection_label
                .set_text(&format!("{count} selected"));
        }
    }

    pub fn set_disk_info(&self, free: u64, total: u64) {
        if total > 0 {
            self.disk_label.set_text(&format!(
                "{} free of {}",
                format::format_size(free),
                format::format_size(total)
            ));
        } else {
            self.disk_label.set_text("");
        }
    }
}

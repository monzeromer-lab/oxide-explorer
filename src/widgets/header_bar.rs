use gtk::prelude::*;

pub struct HeaderBar {
    pub widget: adw::HeaderBar,
    pub back_btn: gtk::Button,
    pub forward_btn: gtk::Button,
    pub up_btn: gtk::Button,
    pub view_toggle: gtk::ToggleButton,
    pub hidden_toggle: gtk::ToggleButton,
    pub breadcrumb_box: gtk::Box,
    pub search_btn: gtk::ToggleButton,
}

impl HeaderBar {
    pub fn new() -> Self {
        let widget = adw::HeaderBar::new();
        widget.set_show_title(false);

        // Navigation buttons
        let back_btn = gtk::Button::from_icon_name("go-previous-symbolic");
        back_btn.set_tooltip_text(Some("Back (Alt+Left)"));
        back_btn.set_sensitive(false);

        let forward_btn = gtk::Button::from_icon_name("go-next-symbolic");
        forward_btn.set_tooltip_text(Some("Forward (Alt+Right)"));
        forward_btn.set_sensitive(false);

        let up_btn = gtk::Button::from_icon_name("go-up-symbolic");
        up_btn.set_tooltip_text(Some("Parent Directory (Alt+Up)"));

        let nav_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        nav_box.add_css_class("linked");
        nav_box.append(&back_btn);
        nav_box.append(&forward_btn);
        nav_box.append(&up_btn);
        widget.pack_start(&nav_box);

        // Breadcrumb in the title area
        let breadcrumb_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        breadcrumb_box.set_hexpand(true);
        widget.set_title_widget(Some(&breadcrumb_box));

        // Hidden files toggle
        let hidden_toggle = gtk::ToggleButton::new();
        hidden_toggle.set_icon_name("view-reveal-symbolic");
        hidden_toggle.set_tooltip_text(Some("Show Hidden Files (Ctrl+H)"));
        widget.pack_end(&hidden_toggle);

        // View toggle (grid/list)
        let view_toggle = gtk::ToggleButton::new();
        view_toggle.set_icon_name("view-list-symbolic");
        view_toggle.set_tooltip_text(Some("Toggle Details View"));
        widget.pack_end(&view_toggle);

        // Search toggle
        let search_btn = gtk::ToggleButton::new();
        search_btn.set_icon_name("edit-find-symbolic");
        search_btn.set_tooltip_text(Some("Filter (Ctrl+F)"));
        widget.pack_end(&search_btn);

        Self {
            widget,
            back_btn,
            forward_btn,
            up_btn,
            view_toggle,
            hidden_toggle,
            breadcrumb_box,
            search_btn,
        }
    }
}

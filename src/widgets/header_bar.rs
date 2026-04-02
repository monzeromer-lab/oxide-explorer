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
    pub zoom_in_btn: gtk::Button,
    pub zoom_out_btn: gtk::Button,
}

impl HeaderBar {
    pub fn new() -> Self {
        let widget = adw::HeaderBar::new();
        // Keep show_title true so the title widget (breadcrumb) is visible
        widget.set_centering_policy(adw::CenteringPolicy::Loose);

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

        // --- Right side buttons ---

        // Hamburger menu
        let primary_menu = gio::Menu::new();

        let view_section = gio::Menu::new();
        view_section.append(Some("Toggle Terminal (F4)"), Some("win.toggle-terminal"));
        view_section.append(Some("Dual Pane (F3)"), Some("win.toggle-dual-pane"));
        view_section.append(Some("Miller Columns (F5)"), Some("win.toggle-miller"));
        primary_menu.append_section(None, &view_section);

        let settings_section = gio::Menu::new();
        settings_section.append(Some("Preferences"), Some("win.preferences"));
        settings_section.append(Some("Keyboard Shortcuts"), Some("win.show-shortcuts"));
        settings_section.append(Some("User Guide"), Some("win.show-guide"));
        primary_menu.append_section(None, &settings_section);

        let about_section = gio::Menu::new();
        about_section.append(Some("About Oxide Explorer"), Some("win.about"));
        primary_menu.append_section(None, &about_section);

        let menu_button = gtk::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        menu_button.set_menu_model(Some(&primary_menu));
        menu_button.set_tooltip_text(Some("Main Menu"));
        widget.pack_end(&menu_button);

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

        // Zoom controls
        let zoom_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        zoom_box.add_css_class("linked");

        let zoom_out_btn = gtk::Button::from_icon_name("zoom-out-symbolic");
        zoom_out_btn.set_tooltip_text(Some("Zoom Out (Ctrl+-)"));
        zoom_box.append(&zoom_out_btn);

        let zoom_in_btn = gtk::Button::from_icon_name("zoom-in-symbolic");
        zoom_in_btn.set_tooltip_text(Some("Zoom In (Ctrl++)"));
        zoom_box.append(&zoom_in_btn);

        widget.pack_end(&zoom_box);

        Self {
            widget,
            back_btn,
            forward_btn,
            up_btn,
            view_toggle,
            hidden_toggle,
            breadcrumb_box,
            search_btn,
            zoom_in_btn,
            zoom_out_btn,
        }
    }
}

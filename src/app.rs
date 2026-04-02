use gtk::prelude::*;
use crate::window::OxideWindow;

pub fn build_app() -> adw::Application {
    let app = adw::Application::builder()
        .application_id("com.oxide.explorer")
        .build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(|app| {
        let win = OxideWindow::new(app);
        win.window.present();
    });

    app
}

fn load_css() {
    let provider = gtk::CssProvider::new();

    // Try loading from installed path, fallback to relative path
    let css_paths = [
        "/usr/share/oxide-explorer/style.css",
        "data/style.css",
    ];

    for path in &css_paths {
        let file = std::path::Path::new(path);
        if file.exists() {
            provider.load_from_path(file);
            gtk::style_context_add_provider_for_display(
                &gtk::gdk::Display::default().expect("Could not get display"),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            return;
        }
    }
}

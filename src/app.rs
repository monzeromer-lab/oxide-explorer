use gtk::prelude::*;
use crate::window::OxideWindow;

pub fn build_app() -> adw::Application {
    let app = adw::Application::builder()
        .application_id("com.oxide.explorer")
        .build();

    app.connect_activate(|app| {
        let win = OxideWindow::new(app);
        win.window.present();
    });

    app
}

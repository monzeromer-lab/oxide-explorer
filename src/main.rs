mod app;
mod models;
mod operations;
mod state;
mod utils;
mod widgets;
mod window;

use gtk::prelude::*;

fn main() {
    env_logger::init();

    let app = app::build_app();
    app.run();
}

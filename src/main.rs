extern crate reqwest;
extern crate ruma_client_api;
extern crate gio;
extern crate gtk;
// extern crate xdg;

mod app;

use app::App;
// use std::fs::File;
// use std::path::Path;

fn main() {
    // let xdg_dirs      = xdg::BaseDirectories::with_prefix("ruma_gtk").unwrap();
    // let data_path     = xdg_dirs.place_data_file("data.yml").unwrap();
    // TODO: Read settings

    // gtk::init().expect("Failed to initialize GTK.");

    let app = App::new();
    app.run();

    // TODO: Save settings
}

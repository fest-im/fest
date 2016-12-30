#![feature(box_syntax)]

// not using this yet because rustfmt doesn't support it:
// https://github.com/rust-lang-nursery/rustfmt/issues/1215
//#![feature(field_init_shorthand)]

extern crate reqwest;
extern crate ruma_client_api;
extern crate gio;
extern crate gtk;
// extern crate xdg;

mod app;
mod matrix_client;

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

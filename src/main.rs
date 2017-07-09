#![feature(box_syntax)]
#![feature(conservative_impl_trait)]

// not using this yet because rustfmt doesn't support it:
// https://github.com/rust-lang-nursery/rustfmt/issues/1215
//#![feature(field_init_shorthand)]

extern crate futures;
extern crate gio;
extern crate gtk;
extern crate ruma_client;
extern crate tokio_core;
extern crate url;
// extern crate xdg;

#[macro_use]
mod util;

mod app;
mod bg_thread;

use app::App;
// use std::fs::File;
// use std::path::Path;

fn main() {
    // let xdg_dirs      = xdg::BaseDirectories::with_prefix("ruma_gtk").unwrap();
    // let data_path     = xdg_dirs.place_data_file("data.yml").unwrap();
    // TODO: Read settings

    let app = App::new();
    app.run();

    // TODO: Save settings
}

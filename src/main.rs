#![feature(box_syntax)]
#![feature(conservative_impl_trait)]
#![feature(clone_closures)]

extern crate futures;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate hyper;
extern crate ruma_client;
extern crate tokio_core;
extern crate url;

#[macro_use]
mod util;

mod app;
mod bg_thread;

use app::App;

fn main() {
    let app = App::new();
    app.run();
}

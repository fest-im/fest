#![feature(box_syntax, conservative_impl_trait, clone_closures, generators, proc_macro)]

extern crate futures_await as futures;
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

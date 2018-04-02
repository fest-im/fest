#![feature(clone_closures, conservative_impl_trait, crate_in_paths, generators, proc_macro)]

extern crate futures_await as futures;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate hyper;
extern crate ruma_client;
extern crate tokio_core;
extern crate url;

#[macro_use]
extern crate log;

#[macro_use]
mod util;

mod app;
mod bg_thread;

fn main() {
    use app::App;

    util::set_up_logging();
    let app = App::new();
    app.run();
}

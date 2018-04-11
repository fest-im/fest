#![feature(crate_in_paths, generators, proc_macro)]

extern crate futures_await as futures;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate hyper;
extern crate hyper_tls;
extern crate ruma_client;
extern crate ruma_events;
extern crate ruma_identifiers;
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

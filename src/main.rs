#![feature(box_syntax, clone_closures, conservative_impl_trait, crate_in_paths, generators,
           proc_macro)]

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

// Re-exports for use in other modules
use app::Command as FrontendCommand;
use bg_thread::{run as run_bg_thread, Command as MatrixCommand};

fn main() {
    use app::App;

    let app = App::new();
    app.run();
}

extern crate ruma_client_api;
extern crate gtk;
//extern crate xdg;

use gtk::prelude::*;
//use std::fs::File;
//use std::path::Path;

fn main() {
    //let xdg_dirs      = xdg::BaseDirectories::with_prefix("γ_notes").unwrap();
    //let data_path     = xdg_dirs.place_data_file("data.yml").unwrap();
    // TODO: Read settings

    gtk::init().expect("Failed to initialize GTK.");

    let builder = gtk::Builder::new_from_file("res/main_window.glade");

    // TODO

    // Set up shutdown callback
    let window: gtk::Window = builder.get_object("main_window")
        .expect("Couldn't find main_window in ui file.");

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Start
    window.show_all();
    gtk::main();

    // TODO: Save settings
}

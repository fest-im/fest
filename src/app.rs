use gio;
use gtk;
use gtk::prelude::*;
use std::{env, sync, time, thread};
use matrix_client;

// TODO: Is this the correct format for GApplication IDs?
const APP_ID: &'static str = "jplatte.ruma_gtk";

pub struct App {
    gtk_app: gtk::Application,
    gtk_builder: gtk::Builder,
    dispatch_rx: sync::mpsc::Receiver<Box<Fn(&gtk::Builder) + Send>>,
    matrix_client_thread_join_handle: thread::JoinHandle<()>,
}

impl App {
    /// Create an App instance
    ///
    /// Requires gtk::init() to have executed successfully.
    pub fn new() -> App {
        let gtk_app = gtk::Application::new(Some(APP_ID), gio::ApplicationFlags::empty())
            .expect("Failed to initialize GtkApplication");

        let gtk_builder = gtk::Builder::new_from_file("res/main_window.glade");

        let gtk_builder2 = gtk_builder.clone();
        gtk_app.connect_activate(move |app| {
            let user_button: gtk::Button = gtk_builder2.get_object("user_button")
                .expect("Couldn't find user_button in ui file.");

            let user_menu: gtk::Popover = gtk_builder2.get_object("user_menu")
                .expect("Couldn't find user_menu in ui file.");

            user_button.connect_clicked(move |_| user_menu.show());

            // Set up shutdown callback
            let window: gtk::Window = gtk_builder2.get_object("main_window")
                .expect("Couldn't find main_window in ui file.");

            let app2 = app.clone();
            window.connect_delete_event(move |_, _| {
                app2.quit();
                Inhibit(false)
            });

            window.set_application(Some(app));
            window.show_all();
        });

        let (dispatch_tx, dispatch_rx) = sync::mpsc::channel::<Box<Fn(&gtk::Builder) + Send>>();

        let matrix_client_thread_join_handle =
            thread::spawn(move || matrix_client::run_client_main(dispatch_tx));

        App {
            gtk_app: gtk_app,
            gtk_builder: gtk_builder,
            dispatch_rx: dispatch_rx,
            matrix_client_thread_join_handle: matrix_client_thread_join_handle,
        }
    }

    pub fn run(self) {
        let args = env::args().collect::<Vec<_>>();
        let args_refs = args.iter().map(|x| &x[..]).collect::<Vec<_>>();

        let dispatch_rx = self.dispatch_rx;
        let gtk_builder = self.gtk_builder;
        gtk::idle_add(move || {
            if let Ok(dispatch_fn) = dispatch_rx.recv_timeout(time::Duration::from_millis(5)) {
                dispatch_fn(&gtk_builder);
            }

            Continue(true)
        });

        self.gtk_app.run(args_refs.len() as i32, &args_refs);

        // Clean up
        self.matrix_client_thread_join_handle.join();
    }
}
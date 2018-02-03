mod launch;

use std::{self, env, thread};
use std::time::Duration;

use futures::{self, Sink};
use gio;
use gio::prelude::*;
use glib;
use gtk;
use gtk::prelude::*;

use bg_thread;

const APP_ID: &'static str = "org.fest-im.fest";

/// State for the main thread.
///
/// It takes care of starting up the application and for loading and accessing the
/// UI.
pub struct App {
    /// GTK Application which runs the main loop.
    gtk_app: gtk::Application,

    /// Used to access the UI elements.
    gtk_builder: gtk::Builder,

    /// Sender for the matrix channel.
    ///
    /// This channel is used to send commands to the background thread.
    command_chan_tx: futures::sink::Wait<futures::sync::mpsc::Sender<bg_thread::Command>>,

    /// Channel receiver which allows to run actions from the matrix connection thread.
    ///
    /// Long polling is required to receive messages from the rooms and so they have to
    /// run in separate threads.  In order to allow those threads to modify the gtk content,
    /// they will send closures to the main thread using this channel.
    ui_dispatch_chan_rx: std::sync::mpsc::Receiver<Box<Fn(&gtk::Builder) + Send>>,

    /// Matrix communication thread join handler used to clean up the tread when
    /// closing the application.
    bg_thread_join_handle: thread::JoinHandle<()>,
}

impl App {
    /// Create an App instance
    pub fn new() -> App {
        let gtk_app = gtk::Application::new(Some(APP_ID), gio::ApplicationFlags::FLAGS_NONE)
            .expect("Failed to initialize GtkApplication");

        // Register gresources
        let register = |gr_bytes| {
            let b = glib::Bytes::from_static(gr_bytes);
            let gresource = gio::Resource::new_from_data(&b).expect("Failed to load gresource.");
            gio::resources_register(&gresource);
        };

        register(include_bytes!("../../res/fest.gresource"));
        register(include_bytes!("../../res/icons/hicolor/icons.gresource"));

        let gtk_builder = gtk::Builder::new_from_resource("/org/fest-im/fest/main_window.glade");

        launch::connect(gtk_app.clone(), gtk_builder.clone());

        let (command_chan_tx, command_chan_rx) = futures::sync::mpsc::channel(1);
        let command_chan_tx = command_chan_tx.wait();

        // Create channel to allow the matrix connection thread to send closures to the main loop.
        let (ui_dispatch_chan_tx, ui_dispatch_chan_rx) = std::sync::mpsc::channel();

        let bg_thread_join_handle =
            thread::spawn(move || bg_thread::run(command_chan_rx, ui_dispatch_chan_tx));

        App {
            gtk_app,
            gtk_builder,
            command_chan_tx,
            ui_dispatch_chan_rx,
            bg_thread_join_handle,
        }
    }

    pub fn run(mut self) {
        // Poll the matrix communication thread channel and run the closures to allow
        // the threads to run actions in the main loop.
        let ui_dispatch_chan_rx = self.ui_dispatch_chan_rx;
        let gtk_builder = self.gtk_builder;
        gtk::idle_add(move || {
            if let Ok(dispatch_fn) = ui_dispatch_chan_rx.recv_timeout(Duration::from_millis(5)) {
                dispatch_fn(&gtk_builder);
            }

            Continue(true)
        });

        // Run the main loop.
        self.gtk_app.run(&env::args().collect::<Vec<_>>());

        // Clean up

        // TODO: This should end the loop in bg_thread::bg_main, but it doesn't seem to...
        // self.command_chan_tx.close().unwrap();
        // So for now, we have this extra variant in Command instead:
        self.command_chan_tx.send(bg_thread::Command::Quit).unwrap();
        self.bg_thread_join_handle.join().unwrap();
    }
}

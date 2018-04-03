mod launch;

use std::{self, env, thread, time::Duration};

use futures::{self, Sink};
use gio::{self, prelude::*};
use glib;
use gtk::{self, prelude::*};
use ruma_identifiers::RoomId;

use crate::bg_thread::{self, MatrixCommand};

const APP_ID: &'static str = "org.fest-im.fest";

pub enum FrontendCommand {
    DisplayTextMessage {
        room_id: RoomId,
        author_name: String,
        // [...]
        message_content: String,
    },
}

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
    backend_chan_tx: futures::sink::Wait<futures::sync::mpsc::Sender<MatrixCommand>>,

    /// Channel receiver which allows to run actions from the matrix connection thread.
    ///
    /// Long polling is required to receive messages from the rooms and so they have to
    /// run in separate threads.  In order to allow those threads to modify the gtk content,
    /// they will send commands to the main thread using this channel.
    frontend_chan_rx: std::sync::mpsc::Receiver<FrontendCommand>,

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

        let (backend_chan_tx, backend_chan_rx) = futures::sync::mpsc::channel(1);

        launch::connect(
            gtk_app.clone(),
            gtk_builder.clone(),
            backend_chan_tx.clone(),
        );

        // Create channel to allow the matrix connection thread to send closures to the main loop.
        let (frontend_chan_tx, frontend_chan_rx) = std::sync::mpsc::channel();

        let bg_thread_join_handle =
            thread::spawn(move || bg_thread::run(backend_chan_rx, frontend_chan_tx));

        App {
            gtk_app,
            gtk_builder,
            backend_chan_tx: backend_chan_tx.wait(),
            frontend_chan_rx,
            bg_thread_join_handle,
        }
    }

    pub fn run(mut self) {
        // Poll the matrix communication thread channel and run the closures to allow
        // the threads to run actions in the main loop.
        let frontend_chan_rx = self.frontend_chan_rx;
        gtk::idle_add(move || {
            if let Ok(cmd) = frontend_chan_rx.recv_timeout(Duration::from_millis(5)) {
                match cmd {
                    FrontendCommand::DisplayTextMessage {
                        room_id,
                        author_name,
                        message_content,
                    } => {
                        // TODO!
                    }
                }
            }

            Continue(true)
        });

        // Run the main loop.
        self.gtk_app.run(&env::args().collect::<Vec<_>>());

        // Clean up

        // TODO: This should end the loop in bg_thread::bg_main, but it doesn't seem to...
        // self.backend_chan_tx.close().unwrap();
        // So for now, we have this extra variant in Command instead:
        self.backend_chan_tx.send(MatrixCommand::Quit).unwrap();
        self.bg_thread_join_handle.join().unwrap();
    }
}

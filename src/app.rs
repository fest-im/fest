use std::{self, env, thread};
use std::time::Duration;

use futures::{self, Sink};
use gio;
use gtk;
use gtk::prelude::*;
use url::Url;

use bg_thread::{self, Command, ConnectionMethod};

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
        let gtk_app = gtk::Application::new(Some(APP_ID), gio::APPLICATION_FLAGS_NONE)
            .expect("Failed to initialize GtkApplication");

        let gtk_builder = gtk::Builder::new_from_file("res/main_window.glade");

        gtk_app.connect_activate(clone!(gtk_builder => move |app| {
            // Set up UI navigation button callbacks
            // It would be nice to use Popover::{popup,popdown} throughout here,
            // but that is only available in gtk 3.22.
            let rd_popover: gtk::Popover = gtk_builder.get_object("room_details_popover")
                .expect("Couldn't find room details popover in ui file.");
            let rd_stack: gtk::Stack = gtk_builder.get_object("rd_stack")
                .expect("Couldn't find room details stack in ui file.");
            let rp_revealer: gtk::Revealer = gtk_builder.get_object("right_pane_revealer")
                .expect("Couldn't find right pane revealer in ui file.");
            let rv_stack: gtk::Stack = gtk_builder.get_object("room_view_stack")
                .expect("Couldn't find room view stack in ui file.");

            rd_popover.connect_hide(clone!(rd_stack => move |_| {
                rd_stack.set_visible_child_name("details");
            }));

            let rd_invite_button: gtk::Button = gtk_builder.get_object("rd_invite_button")
                .expect("Couldn't find room invite button in ui file.");

            rd_invite_button.connect_clicked(clone!(rd_stack => move |_| {
                rd_stack.set_visible_child_name("invite");
            }));

            let rdi_cancel_button: gtk::Button = gtk_builder.get_object("rdi_cancel_button")
                .expect("Couldn't find room invite cancel button in ui file.");

            rdi_cancel_button.connect_clicked(clone!(rd_popover => move |_| {
                rd_popover.hide();
            }));

            let rd_leave_button: gtk::Button = gtk_builder.get_object("rd_leave_button")
                .expect("Couldn't find room leave button in ui file.");

            rd_leave_button.connect_clicked(clone!(rd_stack => move |_| {
                rd_stack.set_visible_child_name("leave");
            }));

            let rdl_stay_button: gtk::Button = gtk_builder.get_object("rdl_stay_button")
                .expect("Couldn't find room leave cancel button in ui file.");

            rdl_stay_button.connect_clicked(clone!(rd_popover => move |_| {
                rd_popover.hide();
            }));

            let rd_pins_button: gtk::ToggleButton = gtk_builder.get_object("rd_pins_button")
                .expect("Couldn't find room pins buttin in ui file.");

            rd_pins_button.connect_toggled(clone!(rd_popover => move |_toggle| {
                // TODO: Toggle room's pinned messages
                rd_popover.hide();
            }));

            let rd_settings_button: gtk::Button = gtk_builder.get_object("rd_settings_button")
                .expect("Couldn't find room settings button in ui file.");

            rd_settings_button.connect_clicked(clone!(rd_popover, rv_stack => move |_| {
                rv_stack.set_visible_child_name("settings");
                rd_popover.hide();
            }));

            let rvs_back_button: gtk::Button = gtk_builder.get_object("rvs_back_button")
                .expect("Couldn't find room settings back button in ui file.");

            rvs_back_button.connect_clicked(clone!(rv_stack => move |_| {
                rv_stack.set_visible_child_name("chat");
            }));

            let u_menu: gtk::PopoverMenu = gtk_builder.get_object("user_menu")
                .expect("Couldn't find user menu in ui file.");

            let u_register_button: gtk::Button = gtk_builder.get_object("u_register_button")
                .expect("Couldn't find user register button in ui file.");

            u_register_button.connect_clicked(clone!(u_menu => move |_| {
                u_menu.open_submenu("new_password");
            }));

            let rp_toggle: gtk::ToggleButton = gtk_builder.get_object("right_pane_toggle")
                .expect("Couldn't find right pane toggle button.");

            rp_toggle.connect_toggled(clone!(rp_revealer => move |toggle| {
                rp_revealer.set_reveal_child(toggle.get_active())
            }));

            // TODO: Add and connect directory view
            let _lp_directory_button: gtk::Button = gtk_builder.get_object("lp_directory_button")
                .expect("Couldn't find directory button in ui file.");

            // Set up composer callbacks
            let ri_popover: gtk::Popover = gtk_builder.get_object("room_interactions_popover")
                .expect("Couldn't find room interactions popover in ui file.");

            let ri_attach_button: gtk::Button = gtk_builder.get_object("ri_attach_button")
                .expect("Couldn't find send attachment button in ui file.");

            ri_attach_button.connect_clicked(clone!(ri_popover => move |_| {
                // TODO: Open file chooser and send attachment
                ri_popover.hide();
            }));

            let ri_video_button: gtk::Button = gtk_builder.get_object("ri_video_button")
                .expect("Couldn't find video call button in ui file.");

            ri_video_button.connect_clicked(clone!(ri_popover => move |_| {
                // TODO: Start video call
                ri_popover.hide();
            }));

            let ri_voice_button: gtk::Button = gtk_builder.get_object("ri_voice_button")
                .expect("Couldn't find voice call button in ui file.");

            ri_voice_button.connect_clicked(clone!(ri_popover => move |_| {
                // TODO: Start voice call
                ri_popover.hide();
            }));

            // Set up shutdown callback
            let window: gtk::Window = gtk_builder.get_object("main_window")
                .expect("Couldn't find main_window in ui file.");

            window.connect_delete_event(clone!(app => move |_, _| {
                app.quit();
                Inhibit(false)
            }));

            // Set up window configuration
            window.set_title("Fest");

            // Associate window with the Application and show it
            window.set_application(Some(app));
            window.present();
        }));

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
        // Convert the args to a Vec<&str>. Application::run requires argv as &[&str]
        // and envd::args() returns an iterator of Strings.
        let args = env::args().collect::<Vec<_>>();
        let args_refs = args.iter().map(|x| &x[..]).collect::<Vec<_>>();

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

        self.command_chan_tx
            .send(Command::Connect {
                homeserver_url: Url::parse("https://matrix.org").unwrap(),
                connection_method: ConnectionMethod::Login {
                    username: "TODO".to_owned(),
                    password: "TODO".to_owned(),
                },
            })
            .unwrap(); // TODO: How to handle background thread crash?

        // Run the main loop.
        self.gtk_app.run(args_refs.len() as i32, &args_refs);

        // Clean up
        self.bg_thread_join_handle.join().unwrap();
    }
}

use gio;
use gtk;
use gtk::prelude::*;
use ruma_client_api::r0 as api;
use std::env;

const APP_ID: &'static str = "jplatte.ruma_gtk";

pub struct App {
    gtk_app: gtk::Application,
    matrix_connection: Option<MatrixConnection>,
}

pub struct MatrixConnection {
    // reqwest::Client
    matrix_access_token: String,
    matrix_user_id: String,
}

impl App {
    /// Create an App instance
    ///
    /// Requires gtk::init() to have executed successfully.
    pub fn new() -> App {
        let gtk_app = gtk::Application::new(Some(APP_ID), gio::ApplicationFlags::empty())
            .expect("Failed to initialize GtkApplication");

        let builder = gtk::Builder::new_from_file("res/main_window.glade");

        gtk_app.connect_activate(move |app| {
            let user_button: gtk::Button = builder.get_object("user_button")
                .expect("Couldn't find user_button in ui file.");

            let user_menu: gtk::Popover = builder.get_object("user_menu")
                .expect("Couldn't find user_menu in ui file.");

            user_button.connect_clicked(move |_| user_menu.show());

            // Set up shutdown callback
            let window: gtk::Window = builder.get_object("main_window")
                .expect("Couldn't find main_window in ui file.");

            window.set_application(Some(app));
            window.show_all();
        });

        App {
            gtk_app: gtk_app,
            matrix_connection: None,
        }
    }

    pub fn run(&self) {
        // Might just be the only way to go from impl Iterator<Item=String> to &[&str]
        let args = env::args().collect::<Vec<_>>();
        let mut args_refs = Vec::<&str>::new();

        for r in &args {
            args_refs.push(r);
        }

        self.gtk_app.run(args_refs.len() as i32, &args_refs);

        /*let client = reqwest::Client::new().unwrap();
        let res = client.post("https://matrix.org/_matrix/client/r0/register?kind=guest")
            .body("{}")
            .send()
            .unwrap()
            .json::<api::account::register::Response>();

        println!("{:?}", res);*/
    }
}
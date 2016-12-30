use gtk;
use reqwest;
use ruma_client_api as api;
use std::sync::mpsc::Sender;

mod reqwest_ext;

pub fn run_client_main(dispatch_tx: Sender<Box<Fn(&gtk::Builder) + Send>>) {
    let client = reqwest::Client::new().unwrap();
    let res = client.post("https://matrix.org/_matrix/client/r0/register?kind=guest")
        .body("{}")
        .send()
        .unwrap()
        .json::<api::r0::account::register::Response>()
        .unwrap();

    dispatch_tx.send(box move |builder| {
        builder.get_object::<gtk::Stack>("user_button_stack")
            .expect("Can't find user_button_stack in ui file.")
            .set_visible_child_name("user_connected_page");

        builder.get_object::<gtk::Label>("display_name_label")
            .expect("Can't find display_name_label in ui file.")
            .set_text("Guest");
    });
}

struct MatrixConnection {
    /// reqwest::Client,
    matrix_access_token: String,
    matrix_user_id: String,
}

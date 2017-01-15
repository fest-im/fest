use gtk;
use ruma_client::Client as RumaClient;
use std::sync::mpsc::Sender;

pub fn run(dispatch_tx: Sender<Box<Fn(&gtk::Builder) + Send>>) {
    let client = RumaClient::new().unwrap();

    // TODO: Register as guest, only when successful do this stuff
    dispatch_tx.send(box move |builder| {
        builder.get_object::<gtk::Stack>("user_button_stack")
            .expect("Can't find user_button_stack in ui file.")
            .set_visible_child_name("user_connected_page");

        builder.get_object::<gtk::Label>("display_name_label")
            .expect("Can't find display_name_label in ui file.")
            .set_text("Guest");
    });
}

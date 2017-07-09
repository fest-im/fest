use std;
use std::error::Error;

use futures::{self, Future, Stream};
use gtk;
use ruma_client::Client as RumaClient;
use tokio_core::reactor::{Core, Handle};
use url::Url;

fn bg_main(
    homeserver: Url,
    core_handle: Handle,
    homeserver_chan_rx: futures::sync::mpsc::Receiver<Url>,
    dispatch_chan_tx: std::sync::mpsc::Sender<Box<Fn(&gtk::Builder) + Send>>,
) -> impl Future<Item = (), Error = Box<Error>> {
    let client = RumaClient::https(&core_handle, homeserver, None).unwrap();

    futures::future::ok(())

    // TODO: background main loop
    // TODO: use loop_fn instead of manually recursing?

    /*dispatch_chan_tx.send(box move |builder| {
        builder
            .get_object::<gtk::Stack>("user_button_stack")
            .expect("Can't find user_button_stack in ui file.")
            .set_visible_child_name("user_connected_page");

        builder
            .get_object::<gtk::Label>("display_name_label")
            .expect("Can't find display_name_label in ui file.")
            .set_text("Guest");
    });*/
}

pub fn run(
    homeserver_chan_rx: futures::sync::mpsc::Receiver<Url>,
    dispatch_chan_tx: std::sync::mpsc::Sender<Box<Fn(&gtk::Builder) + Send>>,
) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    core.run(
        homeserver_chan_rx
            .into_future()
            .map_err(|_| std::sync::mpsc::RecvError.into())
            .and_then(
                move |(opt_url, homeserver_chan_rx)| -> Box<Future<Item = (), Error = Box<Error>>> {
                    if let Some(url) = opt_url {
                        box bg_main(url, handle, homeserver_chan_rx, dispatch_chan_tx)
                    } else {
                        box futures::future::ok(())
                    }
                },
            ),
    ).unwrap();
}

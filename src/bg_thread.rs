use std;

use futures;
use futures::prelude::*;
use futures::future::{self, Future, Loop};
use gtk;
use ruma_client::{self, Client as RumaClient};
use tokio_core::reactor::{Core as TokioCore, Handle as TokioHandle};
use url::Url;

pub enum Command {
    Connect {
        homeserver_url: Url,
        connection_method: ConnectionMethod,
    },
    Quit,
}

#[derive(Clone)]
pub enum ConnectionMethod {
    Login { username: String, password: String },
    Guest,
    //Register,
}

#[derive(Debug)]
enum Error {
    RumaClientError(ruma_client::Error),
    RecvError(std::sync::mpsc::RecvError),
}

impl From<ruma_client::Error> for Error {
    fn from(err: ruma_client::Error) -> Error {
        Error::RumaClientError(err)
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(err: std::sync::mpsc::RecvError) -> Error {
        Error::RecvError(err)
    }
}

#[async]
fn sync(
    tokio_handle: TokioHandle,
    homeserver_url: Url,
    connection_method: ConnectionMethod,
    _ui_dispatch_chan_tx: std::sync::mpsc::Sender<Box<Fn(&gtk::Builder) + Send>>,
) -> Result<(), Error> {
    let client = RumaClient::https(&tokio_handle, homeserver_url, None).unwrap();

    match connection_method {
        ConnectionMethod::Login { username, password } => {
            await!(client.log_in(username, password))?;
        }
        ConnectionMethod::Guest => {
            await!(client.register_guest())?;
        }
    }

    future::loop_fn::<_, (), _, _>((), move |_| {
        use ruma_client::api::r0::sync::sync_events;

        sync_events::call(
            client.clone(),
            sync_events::Request {
                filter: None,
                since: None,
                full_state: None,
                set_presence: None,
                timeout: None,
            },
        ).map(|res| {
            println!("{:?}", res);

            Loop::Continue(())
        })
    });

    Ok(())
}

// TODO: This function should have Result::Error = Error, but we currently never
// return Err(_) anywhere
#[async]
fn bg_main(
    tokio_handle: TokioHandle,
    command_chan_rx: futures::sync::mpsc::Receiver<Command>,
    ui_dispatch_chan_tx: std::sync::mpsc::Sender<Box<Fn(&gtk::Builder) + Send>>,
) -> Result<(), ()> {
    let (sync_cancel_chan_tx, sync_cancel_chan_rx) = futures::sync::oneshot::channel();
    let mut sync_cancel_chan_rx = Some(sync_cancel_chan_rx);

    #[async]
    for command in command_chan_rx {
        match command {
            Command::Connect {
                homeserver_url,
                connection_method,
            } => {
                tokio_handle.spawn(
                    sync(
                        tokio_handle.clone(),
                        homeserver_url,
                        connection_method,
                        ui_dispatch_chan_tx.clone(),
                    ).map_err(|_| ())
                        .select(
                            sync_cancel_chan_rx
                                .take()
                                .expect(
                                    "Switching users after initial connection not yet implemented!",
                                )
                                .map_err(|_| ()),
                        )
                        .then(|_| Ok(())),
                );
            }
            Command::Quit => break,
        }
    }

    let _ = sync_cancel_chan_tx.send(());

    Ok(())

    /*ui_dispatch_chan_tx.send(box move |builder| {
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
    command_chan_rx: futures::sync::mpsc::Receiver<Command>,
    ui_dispatch_chan_tx: std::sync::mpsc::Sender<Box<Fn(&gtk::Builder) + Send>>,
) {
    let mut core = TokioCore::new().unwrap();
    let tokio_handle = core.handle();

    match core.run(bg_main(tokio_handle, command_chan_rx, ui_dispatch_chan_tx)) {
        Ok(_) => {}
        Err(e) => {
            // TODO: Show error message in UI. Quit / restart thread?
            eprintln!("fest: background thread error: {:?}", e);
        }
    };
}

use std;

use futures;
use futures::future::{self, Future, Loop};
use futures::stream::Stream;
use gtk;
use ruma_client::{self, Client as RumaClient};
use tokio_core::reactor::{Core as TokioCore, Handle as TokioHandle};
use url::Url;

pub enum Command {
    Connect {
        homeserver_url: Url,
        connection_method: ConnectionMethod,
    },
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

fn bg_main<'a>(
    tokio_handle: &'a TokioHandle,
    command_chan_rx: futures::sync::mpsc::Receiver<Command>,
    ui_dispatch_chan_tx: std::sync::mpsc::Sender<Box<Fn(&gtk::Builder) + Send>>,
) -> impl Future<Item = (), Error = Error> + 'a {
    future::loop_fn(command_chan_rx, move |command_chan_rx| {
        command_chan_rx
            .into_future()
            // Some sort of error occurred that is not the channel being closed?! Error type is (),
            // so it doesn't even impl Error. Assume this will never happen (for now).
            .map_err(|_| unreachable!())
            .and_then(|(opt_command, command_chan_rx)| match opt_command {
                Some(command) => {
                    let (homeserver_url, connection_method) = match command {
                        Command::Connect { homeserver_url, connection_method }
                            => (homeserver_url, connection_method),
                        //_ => unimplemented!(),
                    };

                    Ok((homeserver_url, connection_method, command_chan_rx))
                }
                None => Err(std::sync::mpsc::RecvError.into()),
            }).and_then(move |(homeserver_url, connection_method, command_chan_rx)| {
                let client = RumaClient::https(tokio_handle, homeserver_url, None).unwrap();

                match connection_method {
                    ConnectionMethod::Login { username, password } => {
                        future::Either::A(client.log_in(username, password))
                    }
                    ConnectionMethod::Guest => future::Either::B(client.register_guest()),
                }.and_then(move |_| {
                    future::loop_fn((), move |_| {
                        use ruma_client::api::r0::sync::sync_events;

                        sync_events::call(client.clone(), sync_events::Request {
                            filter: None,
                            since: None,
                            full_state: None,
                            set_presence: None,
                            timeout: None,
                        }).map(|res| {
                            println!("{:?}", res);

                            Loop::Continue(())
                        })
                    })
                }).map_err(Error::from)
                    //.select(command_chan_rx.into_future())
            })
    })

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

    match core.run(bg_main(&tokio_handle, command_chan_rx, ui_dispatch_chan_tx)) {
        Ok(_) => {}
        Err(e) => {
            // TODO: Show error message in UI. Quit / restart thread?
            eprintln!("fest-im: background thread error: {:?}", e);
        }
    };
}


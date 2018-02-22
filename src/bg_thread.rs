use std;

use futures::{self, future::{self, Future, Loop}, prelude::*};
use ruma_client::{self, Client as RumaClient};
use tokio_core::reactor::{Core as TokioCore, Handle as TokioHandle};
use url::Url;

use crate::FrontendCommand;

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
    _frontend_chan_tx: std::sync::mpsc::Sender<FrontendCommand>,
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
    frontend_chan_tx: std::sync::mpsc::Sender<FrontendCommand>,
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
                        frontend_chan_tx.clone(),
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
}

pub fn run(
    command_chan_rx: futures::sync::mpsc::Receiver<Command>,
    frontend_chan_tx: std::sync::mpsc::Sender<FrontendCommand>,
) {
    let mut core = TokioCore::new().unwrap();
    let tokio_handle = core.handle();

    match core.run(bg_main(tokio_handle, command_chan_rx, frontend_chan_tx)) {
        Ok(_) => {}
        Err(e) => {
            // TODO: Show error message in UI. Quit / restart thread?
            eprintln!("fest: background thread error: {:?}", e);
        }
    };
}

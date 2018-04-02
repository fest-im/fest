use std;
use std::cell::RefCell;
use std::collections::hash_map::{Entry as HashMapEntry, HashMap};
use std::rc::Rc;

use futures::{self, future::{self, Future, Loop}, prelude::*};
use ruma_client;
use tokio_core;
use url::Url;

use crate::app::FrontendCommand;

// We refer to users with numerical IDs (a simple counter) internally, because
// using the matrix user id to refer to users would involve a roundtrip to the
// homeserver when registering as a guest.
pub type InternalUserId = u32;

pub enum MatrixCommand {
    Connect {
        homeserver_url: Url,
        connection_method: ConnectionMethod,
    },
    Disconnect(InternalUserId),
    FetchDirectory(InternalUserId),
    Quit,
}

pub enum ConnectionMethod {
    Login { username: String, password: String },
    Guest,
    //Register,
}

pub struct UserMetadata {
    homeserver: Option<String>,
    username: Option<String>,
    display_name: Option<String>,
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
    tokio_handle: tokio_core::reactor::Handle,
    homeserver_url: Url,
    connection_method: ConnectionMethod,
    _user_metadata: Rc<RefCell<UserMetadata>>,
    _frontend_chan_tx: std::sync::mpsc::Sender<FrontendCommand>,
) -> Result<(), Error> {
    let client = ruma_client::Client::https(&tokio_handle, homeserver_url, None).unwrap();

    match connection_method {
        ConnectionMethod::Login { username, password } => {
            await!(client.log_in(username, password))?;
        }
        ConnectionMethod::Guest => {
            await!(client.register_guest())?;
        }
    }

    // TODO: Fill in user metadata

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

    unreachable!()
}

// TODO: This function should have Result::Error = Error, but we currently never
// return Err(_) anywhere
#[async]
fn bg_main(
    tokio_handle: tokio_core::reactor::Handle,
    backend_chan_rx: futures::sync::mpsc::Receiver<MatrixCommand>,
    frontend_chan_tx: std::sync::mpsc::Sender<FrontendCommand>,
) -> Result<(), ()> {
    let mut next_user_id = 0;
    let mut sync_cancel_chan_txs = HashMap::new();
    let mut user_metadata_map = HashMap::new();

    #[async]
    for command in backend_chan_rx {
        match command {
            MatrixCommand::Connect {
                homeserver_url,
                connection_method,
            } => {
                let (sync_cancel_chan_tx, sync_cancel_chan_rx) = futures::sync::oneshot::channel();
                sync_cancel_chan_txs.insert(next_user_id, sync_cancel_chan_tx);

                let user_metadata = Rc::new(RefCell::new(UserMetadata {
                    // TODO: Can / should we obtain this another way? It is
                    // probably possible to connect to a homeserver using its
                    // IP or a secondary hostname.
                    homeserver: homeserver_url.host_str().map(|host| host.to_owned()),
                    username: match connection_method {
                        ConnectionMethod::Login { ref username, .. } => Some(username.clone()),
                        ConnectionMethod::Guest => None,
                    },
                    display_name: None,
                }));
                user_metadata_map.insert(next_user_id, user_metadata.clone());

                tokio_handle.spawn(
                    sync(
                        tokio_handle.clone(),
                        homeserver_url,
                        connection_method,
                        user_metadata,
                        frontend_chan_tx.clone(),
                    ).map_err(|_| ())
                        .select(sync_cancel_chan_rx.map_err(|_| ()))
                        .then(|_| Ok(())),
                );

                next_user_id += 1;
            }
            MatrixCommand::Disconnect(user_id) => {
                match sync_cancel_chan_txs.entry(user_id) {
                    HashMapEntry::Vacant(_) => {
                        // TODO: Log an error
                    }
                    HashMapEntry::Occupied(o) => {
                        let (_, sync_cancel_chan_tx) = o.remove_entry();
                        let _ = sync_cancel_chan_tx.send(());
                    }
                }
            }
            MatrixCommand::FetchDirectory(_) => unimplemented!(),
            MatrixCommand::Quit => break,
        }
    }

    for (_, sync_cancel_chan_tx) in sync_cancel_chan_txs {
        let _ = sync_cancel_chan_tx.send(());
    }

    Ok(())
}

pub fn run(
    backend_chan_rx: futures::sync::mpsc::Receiver<MatrixCommand>,
    frontend_chan_tx: std::sync::mpsc::Sender<FrontendCommand>,
) {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let tokio_handle = core.handle();

    match core.run(bg_main(tokio_handle, backend_chan_rx, frontend_chan_tx)) {
        Ok(_) => {}
        Err(e) => {
            // TODO: Show error message in UI. Quit / restart thread?
            eprintln!("fest: background thread error: {:?}", e);
        }
    };
}

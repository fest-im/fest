use std::{
    self,
    cell::RefCell,
    collections::hash_map::{Entry as HashMapEntry, HashMap},
    rc::Rc,
};

use futures::{
    self,
    prelude::{async, await},
    Future,
    Stream,
};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use ruma_client::{self, api::r0};
use ruma_events::{
    room::message::{MessageEventContent, MessageType, TextMessageEventContent},
    EventType,
};
use ruma_identifiers::RoomId;
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
    // This is not a UserSpecificCommand because it deletes user data rather
    // than just accessing it.
    Disconnect(InternalUserId),
    UserSpecificCommand {
        user_id: InternalUserId,
        command: UserSpecificCommand,
    },
    Quit,
}

pub enum UserSpecificCommand {
    FetchDirectory,
    SendTextMessage {
        room_id: RoomId,
        message_content: String,
    },
    // [...]
}

pub enum ConnectionMethod {
    Login { username: String, password: String },
    Guest,
    //Register,
}

pub struct UserData {
    client: ruma_client::Client<HttpsConnector<HttpConnector>>,
    homeserver: Option<String>,
    username: Option<String>,
    display_name: Option<String>,
}

#[async]
fn sync(
    connection_method: ConnectionMethod,
    user_data: Rc<RefCell<UserData>>,
    _frontend_chan_tx: std::sync::mpsc::Sender<FrontendCommand>,
) -> Result<(), ()> {
    let client = user_data.borrow().client.clone();

    match connection_method {
        ConnectionMethod::Login { username, password } => {
            // TODO: Set the last param (device_id) to Some(_)
            await!(client.log_in(username.clone(), password, None)).map_err(|e| {
                error!("Failed to log in as {}: {:?}", username, e);
            })?;
        }
        ConnectionMethod::Guest => {
            await!(client.register_guest()).map_err(|e| {
                error!("Failed to log in as guest: {:?}", e);
            })?;
        }
    }

    // TODO: Fill in user metadata

    #[async]
    for event in client.sync(None, None, false).map_err(|e| {
        error!("Error in sync_events: {:?}", e);
    }) {
        trace!("synchronization response: {:?}", event);
    }

    unreachable!()
}

#[async]
fn fetch_directory(
    _user_data: Rc<RefCell<UserData>>,
    _frontend_chan_tx: std::sync::mpsc::Sender<FrontendCommand>,
) -> Result<(), ()> {
    unimplemented!()
}

#[async]
fn send_text_message(
    user_data: Rc<RefCell<UserData>>,
    frontend_chan_tx: std::sync::mpsc::Sender<FrontendCommand>,
    room_id: RoomId,
    message_content: String,
) -> Result<(), ()> {
    // TODO: Indicate that the server hasn't received the message yet?
    // TODO: Handle channel send errors?
    let _ = frontend_chan_tx.send(FrontendCommand::DisplayTextMessage {
        room_id: room_id.clone(),
        author_name: user_data.borrow().username.clone().ok_or_else(|| {
            error!("send_text_message: UserData::username not set!");
        })?,
        message_content: message_content.clone(),
    });

    await!(r0::send::send_message_event::call(
        user_data.borrow().client.clone(),
        r0::send::send_message_event::Request {
            room_id: room_id.clone(),
            event_type: EventType::RoomMessage,
            txn_id: "1".to_owned(),
            data: MessageEventContent::Text(TextMessageEventContent {
                body: message_content,
                msgtype: MessageType::Text,
            }),
        }
    ))
    .map(|_| {})
    .map_err(|e| {
        error!("Sending a text message to {} failed: {:?}", room_id, e);
    })
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
    let mut user_data_map = HashMap::new();

    #[async]
    for command in backend_chan_rx {
        match command {
            MatrixCommand::Connect {
                homeserver_url,
                connection_method,
            } => {
                let (sync_cancel_chan_tx, sync_cancel_chan_rx) = futures::sync::oneshot::channel();
                sync_cancel_chan_txs.insert(next_user_id, sync_cancel_chan_tx);

                let client = ruma_client::Client::https(homeserver_url.clone(), None).unwrap();

                let user_data = Rc::new(RefCell::new(UserData {
                    client,
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
                user_data_map.insert(next_user_id, user_data.clone());

                tokio_handle.spawn(
                    sync(connection_method, user_data, frontend_chan_tx.clone())
                        .select(sync_cancel_chan_rx.map_err(|e| {
                            error!("some error occured with a rx sync channel: {}", e);
                        }))
                        .then(|_| {
                            // Sync never terminates successfully, so we only reach this when an
                            // error occurs (which is logged inside sync), the sync is cancelled
                            // or receiving a message from sync_cancel_chan_rx failed (logged in
                            // map_err above). We don't want to do anything in any of those cases.
                            Ok(())
                        }),
                );

                next_user_id += 1;
            }
            MatrixCommand::Disconnect(user_id) => match sync_cancel_chan_txs.entry(user_id) {
                HashMapEntry::Vacant(_) => {
                    error!("Tried to disconnect unknown user with user_id {}!", user_id);
                }
                HashMapEntry::Occupied(o) => {
                    let (_, sync_cancel_chan_tx) = o.remove_entry();
                    let _ = sync_cancel_chan_tx.send(());
                }
            },
            MatrixCommand::UserSpecificCommand { user_id, command } => match user_data_map
                .get(&user_id)
            {
                Some(user_data) => match command {
                    UserSpecificCommand::FetchDirectory => {
                        tokio_handle
                            .spawn(fetch_directory(user_data.clone(), frontend_chan_tx.clone()));
                    }
                    UserSpecificCommand::SendTextMessage {
                        room_id,
                        message_content,
                    } => {
                        tokio_handle.spawn(send_text_message(
                            user_data.clone(),
                            frontend_chan_tx.clone(),
                            room_id,
                            message_content,
                        ));
                    }
                },
                None => {
                    error!(
                        "UserSpecificCommand requested for unknown user with user_id {}",
                        user_id
                    );
                }
            },
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
        Err(()) => {
            // TODO: Show error message in UI. Quit / restart thread?
            error!("background thread crashed!");
        }
    };
}

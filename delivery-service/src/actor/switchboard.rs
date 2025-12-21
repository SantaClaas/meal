use std::{collections::HashMap, sync::Arc};

use axum::extract::ws;
use tokio::sync::{mpsc, oneshot};

use crate::actor::{client, web_socket};

struct ClientNotFound;

enum Message {
    Send {
        client_id: Arc<str>,
        message: Arc<[u8]>,
        response: oneshot::Sender<Result<(), ClientNotFound>>,
    },
    Connect {
        client_id: Arc<str>,
        web_socket: ws::WebSocket,
    },
}

struct Switchboard {
    receiver: mpsc::Receiver<Message>,
    clients: HashMap<Arc<str>, client::Handle>,
}

impl Switchboard {
    fn new(receiver: mpsc::Receiver<Message>) -> Self {
        Self {
            receiver,
            clients: HashMap::new(),
        }
    }

    async fn handle_message(&mut self, message: Message) {
        match message {
            Message::Send {
                client_id,
                message,
                response,
            } => {
                //TODO might need to store the message for the client to receive later
                let Some(client) = self.clients.get_mut(&client_id) else {
                    // We don't care if they stopped waiting for the response
                    _ = response.send(Err(ClientNotFound));
                    return;
                };

                let Err(client::HandleError::Closed) = client.send_message(message).await else {
                    _ = response.send(Ok(()));
                    return;
                };

                // TODO client is not active. Store message for the client to receive later or send push notification
                _ = response.send(Ok(()));
            }
            Message::Connect {
                client_id,
                web_socket,
            } => {
                let socket = web_socket::Handle::new(web_socket);

                let client = self
                    .clients
                    .entry(client_id.clone())
                    .or_insert_with(|| client::Handle::new(client_id.clone()));

                let Err(client::HandleError::Closed) = client.add_socket(socket.clone()).await
                else {
                    tracing::debug!(
                        "[Switchboard] Client {} added socket {}",
                        socket.id,
                        client_id
                    );
                    return;
                };

                tracing::debug!(
                    "[Switchboard] Client actor {} is dead. Rebirthing client.",
                    client_id
                );
                let client = client.rebirth();
                self.clients.insert(client_id.clone(), client.clone());

                let socket_id = socket.id.clone();
                if let Err(client::HandleError::Closed) = client.add_socket(socket).await {
                    tracing::error!("[Switchboard] Just rebirthed client actor is already dead")
                }

                tracing::debug!(
                    "[Switchboard] Client {} added socket {}",
                    client_id,
                    socket_id
                );
            }
        }
    }

    async fn run(mut self) {
        loop {
            match self.receiver.recv().await {
                Some(message) => self.handle_message(message).await,
                None => {
                    tracing::debug!("[Switchboard] Actor send channel closed");
                    return;
                }
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Handle {
    sender: mpsc::Sender<Message>,
}

pub(crate) enum SendMessageError {
    Closed,
    ClientNotFound,
}

#[derive(Debug)]
pub(crate) enum ConnectError {
    Closed,
}

impl From<ClientNotFound> for SendMessageError {
    fn from(_: ClientNotFound) -> Self {
        Self::ClientNotFound
    }
}

impl Default for Handle {
    fn default() -> Self {
        Self::new()
    }
}

impl Handle {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let actor = Switchboard::new(receiver);
        tokio::spawn(actor.run());
        Self { sender }
    }

    pub(crate) async fn send_message(
        &self,
        client_id: Arc<str>,
        message: Arc<[u8]>,
    ) -> Result<(), SendMessageError> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Message::Send {
                client_id,
                message,
                response: sender,
            })
            .await
            .map_err(|_| SendMessageError::Closed)?;

        receiver.await.map_err(|_| SendMessageError::Closed)??;
        Ok(())
    }

    pub(crate) async fn add_connection(
        &self,
        client_id: Arc<str>,
        web_socket: ws::WebSocket,
    ) -> Result<(), ConnectError> {
        self.sender
            .send(Message::Connect {
                client_id,
                web_socket,
            })
            .await
            .map_err(|_| ConnectError::Closed)
    }
}

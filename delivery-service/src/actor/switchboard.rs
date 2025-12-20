use std::{collections::HashMap, sync::Arc};

use axum::extract::ws;
use tokio::sync::{mpsc, oneshot};

use crate::actor::{user, web_socket};

struct UserNotFound;

enum Message {
    Send {
        user_id: Arc<str>,
        message: Arc<[u8]>,
        response: oneshot::Sender<Result<(), UserNotFound>>,
    },
    Connect {
        user_id: Arc<str>,
        web_socket: ws::WebSocket,
    },
}

struct Switchboard {
    receiver: mpsc::Receiver<Message>,
    users: HashMap<Arc<str>, user::Handle>,
}

impl Switchboard {
    fn new(receiver: mpsc::Receiver<Message>) -> Self {
        Self {
            receiver,
            users: HashMap::new(),
        }
    }

    async fn handle_message(&mut self, message: Message) {
        match message {
            Message::Send {
                user_id,
                message,
                response,
            } => {
                //TODO might need to store the message for the user to receive later
                let Some(user) = self.users.get_mut(&user_id) else {
                    // We don't care if they stopped waiting for the response
                    _ = response.send(Err(UserNotFound));
                    return;
                };

                let Err(user::HandleError::Closed) = user.send_message(message).await else {
                    _ = response.send(Ok(()));
                    return;
                };

                // TODO user is not active. Store message for the user to receive later or send push notification
                _ = response.send(Ok(()));
            }
            Message::Connect {
                user_id,
                web_socket,
            } => {
                let socket = web_socket::Handle::new(web_socket);

                let user = self
                    .users
                    .entry(user_id.clone())
                    .or_insert_with(|| user::Handle::new(user_id.clone()));

                let Err(user::HandleError::Closed) = user.add_socket(socket.clone()).await else {
                    return;
                };

                tracing::debug!(
                    "[Switchboard] User actor {} is dead. Rebirthing user.",
                    user_id
                );
                let user = user.rebirth();
                self.users.insert(user_id, user.clone());

                if let Err(user::HandleError::Closed) = user.add_socket(socket).await {
                    tracing::error!("[Switchboard] Just rebirthed user actor is already dead")
                }
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
    UserNotFound,
}

pub(crate) enum ConnectError {
    Closed,
}

impl From<UserNotFound> for SendMessageError {
    fn from(_: UserNotFound) -> Self {
        Self::UserNotFound
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

    pub(in crate::actor) async fn send_message(
        &self,
        user_id: Arc<str>,
        message: Arc<[u8]>,
    ) -> Result<(), SendMessageError> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Message::Send {
                user_id,
                message,
                response: sender,
            })
            .await
            .map_err(|_| SendMessageError::Closed)?;

        receiver.await.map_err(|_| SendMessageError::Closed)??;
        Ok(())
    }

    pub async fn add_connection(
        &self,
        user_id: Arc<str>,
        web_socket: ws::WebSocket,
    ) -> Result<(), ConnectError> {
        self.sender
            .send(Message::Connect {
                user_id,
                web_socket,
            })
            .await
            .map_err(|_| ConnectError::Closed)
    }
}

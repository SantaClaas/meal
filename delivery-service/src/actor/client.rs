use std::{collections::HashMap, ops::ControlFlow, sync::Arc};

use tokio::{sync::mpsc, task::JoinSet};

use crate::actor::web_socket;

enum Message {
    Send(Arc<[u8]>),
    AddConnection(web_socket::Handle),
}

struct Client {
    id: Arc<str>,
    receiver: mpsc::Receiver<Message>,
    sockets: HashMap<Arc<str>, web_socket::Handle>,
}

impl Client {
    fn new(id: Arc<str>, receiver: mpsc::Receiver<Message>) -> Self {
        Self {
            id,
            receiver,
            sockets: HashMap::new(),
        }
    }

    async fn handle_message(&mut self, message: Message) -> ControlFlow<()> {
        match message {
            Message::Send(data) => {
                let mut sends = JoinSet::new();
                for socket in self.sockets.values().cloned() {
                    tracing::debug!(
                        "[Clients/{}] Sending message to socket {}",
                        self.id,
                        socket.id
                    );
                    let data = data.clone();
                    sends
                        .spawn(async move { (socket.id.clone(), socket.send_message(data).await) });
                }

                let results = sends.join_all().await;
                for (id, result) in results {
                    let Err(web_socket::HandleError::SocketClosed) = result else {
                        continue;
                    };

                    tracing::debug!(
                        "[Clients/{}] Removing closed websocket receiver {}",
                        self.id,
                        id
                    );

                    self.sockets.remove(&id);
                }

                // If there are no more sockets, remove the client
                if self.sockets.is_empty() {
                    tracing::debug!(
                        "[Clients/{}] No more sockets. Stopping client actor",
                        self.id
                    );
                    return ControlFlow::Break(());
                }

                ControlFlow::Continue(())
            }
            Message::AddConnection(handle) => {
                self.sockets.insert(handle.id.clone(), handle);
                ControlFlow::Continue(())
            }
        }
    }

    async fn run(mut self) {
        loop {
            match self.receiver.recv().await {
                Some(message) => {
                    if self.handle_message(message).await.is_break() {
                        return;
                    }
                }
                None => {
                    tracing::debug!("[Clients/{}] Actor send channel closed", self.id);
                    return;
                }
            }
        }
    }
}

#[derive(Clone)]
pub(in crate::actor) struct Handle {
    pub(in crate::actor) id: Arc<str>,
    sender: mpsc::Sender<Message>,
}

pub(in crate::actor) enum HandleError {
    Closed,
}

impl Handle {
    pub(in crate::actor) fn new(id: Arc<str>) -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let actor = Client::new(id, receiver);
        let id = actor.id.clone();
        tokio::spawn(actor.run());

        Self { id, sender }
    }

    /// Revive is the same entity but this creates a new instance so it is a rebirth
    pub(in crate::actor) fn rebirth(&self) -> Self {
        Self::new(self.id.clone())
    }

    pub(in crate::actor) async fn add_socket(
        &self,
        socket: web_socket::Handle,
    ) -> Result<(), HandleError> {
        self.sender
            .send(Message::AddConnection(socket))
            .await
            .map_err(|_| HandleError::Closed)
    }

    pub(in crate::actor) async fn send_message(
        &self,
        data: impl Into<Arc<[u8]>>,
    ) -> Result<(), HandleError> {
        self.sender
            .send(Message::Send(data.into()))
            .await
            .map_err(|_| HandleError::Closed)
    }
}

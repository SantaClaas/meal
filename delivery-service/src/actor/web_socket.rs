use std::{ops::ControlFlow, sync::Arc};

use axum::extract::ws::{self};
use nanoid::nanoid;
use tokio::sync::mpsc;

enum Message {
    Send(Arc<[u8]>),
}
struct WebSocket {
    id: Arc<str>,
    receiver: mpsc::Receiver<Message>,
    socket: ws::WebSocket,
}

impl WebSocket {
    fn new(receiver: mpsc::Receiver<Message>, socket: ws::WebSocket) -> Self {
        Self {
            id: nanoid!().into(),
            receiver,
            socket,
        }
    }

    async fn handle_message(&mut self, Message::Send(data): Message) -> Result<(), axum::Error> {
        // Am I just copying unnecessarily converting from Rc to Vec when I could send a Vec in the first place?
        let message = ws::Message::Binary(Vec::from(data.as_ref()));
        self.socket.send(message).await?;

        Ok(())
    }

    fn handle_socket_message(&mut self, message: ws::Message) -> ControlFlow<()> {
        match message {
            ws::Message::Close(_) => {
                tracing::debug!("[WebSockets/{}] Websocket closed", self.id);
                ControlFlow::Break(())
            }
            other => {
                tracing::debug!(
                    "[WebSockets/{}] Unexpected Websocket message: {:?}",
                    self.id,
                    other
                );
                ControlFlow::Continue(())
            }
        }
    }

    async fn run(mut self) -> Result<(), axum::Error> {
        loop {
            tokio::select! {
                message = self.receiver.recv() => match message {
                    Some(message) => self.handle_message(message).await?,
                    None => {
                        tracing::debug!("[WebSockets/{}] Actor send channel closed", self.id);
                        return Ok(());
                    },
                },
                message = self.socket.recv() => match message {
                    Some(Ok(message)) => if self.handle_socket_message(message).is_break() { return Ok(()); },
                    Some(Err(error)) => {
                        tracing::debug!("[WebSockets/{}] websocket error: {}", self.id, error);
                        return Err(error.into());
                    },
                    None => {
                        tracing::debug!("[WebSockets/{}] client closed websocket", self.id);
                        return Ok(());
                    }
                },
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
    SocketClosed,
}

impl Handle {
    pub(in crate::actor) fn new(socket: ws::WebSocket) -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let actor = WebSocket::new(receiver, socket);
        let id = actor.id.clone();

        tokio::spawn(actor.run());

        Self { id, sender }
    }

    pub(in crate::actor) async fn send_message(
        &self,
        data: impl Into<Arc<[u8]>>,
    ) -> Result<(), HandleError> {
        let data = data.into();
        self.sender
            .send(Message::Send(data))
            .await
            .map_err(|_| HandleError::SocketClosed)
    }
}

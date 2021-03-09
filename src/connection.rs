use crate::master::Services;
use crate::proto::Reply;
use futures_util::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;
use tracing::{error, info, instrument, warn};

#[derive(Debug)]
pub(crate) struct Client<T: AsyncRead + AsyncWrite + Unpin> {
    ws: WebSocketStream<T>,
    addr: String,
    srv: Services,
}

macro_rules! reunite {
    ($field:expr, $in:expr, $out:expr) => {
        $field = match $in.reunite($out) {
            Ok(x) => x,
            Err(x) => {
                error!("Could not reunite streams: {}", x);
                return;
            }
        };
    };
}

impl<T: AsyncRead + AsyncWrite + Unpin> Client<T> {
    pub(crate) fn new(ws: WebSocketStream<T>, addr: String, srv: Services) -> Client<T> {
        Client { ws, addr, srv }
    }

    #[instrument(name = "client", skip(self))]
    pub(crate) async fn start(mut self, address: &str) {
        info!("Client connected");
        let (wsout, mut wsin) = self.ws.split();
        while let Some(msg) = wsin.next().await {
            match msg {
                Ok(Message::Text(msg)) => match Reply::parse(&msg).await {
                    Ok(Reply::Handshake(magic, version)) => {
                        // TODO: la cosa ovvia, magari macro helper
                    }
                    Ok(_) => {
                        warn!("Wrong message while handshaking");
                        break;
                    }
                    Err(x) => {
                        warn!("Invalid message from client: {}", x);
                        break;
                    }
                },
                Err(x) => {
                    warn!("Connection error: {}", x);
                    break;
                }
                _ => {}
            }
        }
        reunite!(self.ws, wsin, wsout);
        self.stop().await;
    }

    async fn stop(mut self) {
        if let Err(x) = self.ws.close(None).await {
            warn!("Could not close connection gracefully: {}", x);
        }
    }
}

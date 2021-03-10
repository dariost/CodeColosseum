use crate::game;
use crate::master::Services;
use crate::proto::{self, Reply, Request};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Error as TsError;
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

macro_rules! send {
    ($out:expr, $reply:expr) => {
        let msg = match Reply::forge(&$reply) {
            Ok(x) => x,
            Err(x) => {
                error!("Cannot forge reply: {}", x);
                break;
            }
        };
        if let Err(x) = $out.send(Message::Text(msg)).await {
            warn!("Cannot send reply: {}", x);
            break;
        }
    };
}

impl<T: AsyncRead + AsyncWrite + Unpin> Client<T> {
    pub(crate) fn new(ws: WebSocketStream<T>, addr: String, srv: Services) -> Client<T> {
        Client { ws, addr, srv }
    }

    #[instrument(name = "client", skip(self))]
    pub(crate) async fn start(mut self, address: &str) {
        info!("Client connected");
        let (mut wsout, mut wsin) = self.ws.split();
        while let Some(msg) = wsin.next().await {
            match msg {
                Ok(Message::Text(msg)) => match Request::parse(&msg) {
                    Ok(Request::Handshake { magic, version }) => {
                        match Reply::forge(&Reply::Handshake {
                            magic: proto::MAGIC.to_string(),
                            version: proto::VERSION,
                        }) {
                            Ok(msg) => {
                                if let Err(x) = wsout.send(Message::Text(msg)).await {
                                    warn!("Cannot send reply: {}", x);
                                } else if magic == proto::MAGIC && version == proto::VERSION {
                                    reunite!(self.ws, wsin, wsout);
                                    return self.main().await;
                                }
                            }
                            Err(x) => error!("Cannot forge handshake reply: {}", x),
                        }
                    }
                    Ok(_) => warn!("Wrong message while handshaking"),
                    Err(x) => warn!("Invalid request from client: {}", x),
                },
                Err(x) => warn!("Connection error: {}", x),
                _ => continue,
            }
            break;
        }
        reunite!(self.ws, wsin, wsout);
        return self.stop().await;
    }

    async fn main(mut self) {
        let (mut wsout, mut wsin) = self.ws.split();
        loop {
            let msg = match wsin.next().await {
                Some(Ok(Message::Text(x))) => x,
                Some(Ok(_)) => continue,
                Some(Err(x)) => {
                    warn!("Connection error: {}", x);
                    break;
                }
                None => break,
            };
            let req = match Request::parse(&msg) {
                Ok(x) => x,
                Err(x) => {
                    warn!("Invalid request from client: {}", x);
                    break;
                }
            };
            match req {
                Request::GameList => {
                    let (tx, rx) = oneshot::channel();
                    if let Err(_) = self.srv.game.send(game::Command::GetList(tx)).await {
                        error!("Cannot forward request to game::Command::GetList");
                        break;
                    }
                    let games = match rx.await {
                        Ok(x) => x,
                        Err(x) => {
                            error!("Cannot get reply from game::Command::GetList: {}", x);
                            break;
                        }
                    };
                    send!(wsout, Reply::GameList { games });
                }
                Request::GameDescription { name } => {
                    let (tx, rx) = oneshot::channel();
                    if let Err(_) = self
                        .srv
                        .game
                        .send(game::Command::GetDescription(tx, name))
                        .await
                    {
                        error!("Cannot forward request to game::Command::GetDescription");
                        break;
                    }
                    let description = match rx.await {
                        Ok(x) => x,
                        Err(x) => {
                            error!("Cannot get reply from game::Command::GetDescription: {}", x);
                            break;
                        }
                    };
                    send!(wsout, Reply::GameDescription { description });
                }
                _ => {
                    warn!("Request not valid for current state: {:?}", req);
                    break;
                }
            };
        }
        reunite!(self.ws, wsin, wsout);
        return self.stop().await;
    }

    async fn stop(mut self) {
        match self.ws.close(None).await {
            Ok(()) | Err(TsError::ConnectionClosed) => info!("Client disconnected"),
            Err(x) => warn!("Could not close connection gracefully: {}", x),
        }
    }
}

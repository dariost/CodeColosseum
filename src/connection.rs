use crate::master::Services;
use crate::proto::{self, Reply, Request};
use crate::{game, lobby};
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

macro_rules! oneshot_reply {
    ($srv:expr, $cmd:expr) => {{
        let (tx, rx) = oneshot::channel();
        if let Err(_) = $srv.send($cmd(tx)).await {
            error!("Cannot forward request to {}", stringify!($cmd));
            break;
        }
        match rx.await {
            Ok(x) => x,
            Err(x) => {
                error!("Cannot get reply from {}: {}", stringify!($cmd), x);
                break;
            }
        }
    }};
    ($srv:expr, $cmd:expr, $($arg:tt)+) => {{
        let (tx, rx) = oneshot::channel();
        if let Err(_) = $srv.send($cmd(tx, $($arg)+)).await {
            error!("Cannot forward request to {}", stringify!($cmd));
            break;
        }
        match rx.await {
            Ok(x) => x,
            Err(x) => {
                error!("Cannot get reply from {}: {}", stringify!($cmd), x);
                break;
            }
        }
    }};
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
                    let games = oneshot_reply!(self.srv.game, game::Command::GetList);
                    send!(wsout, Reply::GameList { games });
                }
                Request::GameDescription { name } => {
                    let description =
                        oneshot_reply!(self.srv.game, game::Command::GetDescription, name);
                    send!(wsout, Reply::GameDescription { description });
                }
                Request::LobbyList => {
                    let info = oneshot_reply!(self.srv.lobby, lobby::Command::GetList);
                    send!(wsout, Reply::LobbyList { info });
                }
                Request::GameNew {
                    name,
                    game,
                    params,
                    args,
                    hidden,
                } => {
                    let id = oneshot_reply!(
                        self.srv.lobby,
                        lobby::Command::NewGame,
                        name,
                        game,
                        params,
                        args,
                        hidden
                    );
                    send!(wsout, Reply::GameNew { id });
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

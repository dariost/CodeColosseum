use crate::master::Services;
use crate::proto::{self, Reply, Request};
use crate::tuning::QUEUE_BUFFER;
use crate::{game, lobby};
use futures_util::sink::Sink;
use futures_util::stream::Stream;
use futures_util::{SinkExt, StreamExt};
use std::fmt::Display;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::select;
use tokio::sync::{mpsc, oneshot};
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
    ($out:expr, $reply:expr) => {{
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
    }};
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
                Request::LobbySubscribe => {
                    if let Err(()) = Self::lobby(&mut wsin, &mut wsout, &self.srv).await {
                        break;
                    }
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

    async fn lobby<X: Sink<Message> + Unpin, Y: Stream<Item = Result<Message, TsError>> + Unpin>(
        wsin: &mut Y,
        wsout: &mut X,
        srv: &Services,
    ) -> Result<(), ()>
    where
        <X as Sink<Message>>::Error: Display,
    {
        let (otx, orx) = oneshot::channel();
        let (tx, mut rx) = mpsc::channel(QUEUE_BUFFER);
        srv.lobby
            .send(lobby::Command::Subscribe(otx, tx))
            .await
            .map_err(|x| error!("Lobby is unreachable: {}", x))?;
        let id = orx.await.map_err(|_| error!("Lobby did not respond"))?;
        loop {
            select! {
                msg = wsin.next() => {
                    let msg = match msg {
                        Some(Ok(Message::Text(x))) => x,
                        Some(_) => continue,
                        None => {
                            srv.lobby.send(lobby::Command::Unsubscribe(id.clone())).await
                               .map_err(|x| error!("Lobby is unreachable: {}", x))?;
                            break;
                        }
                    };
                    match Request::parse(&msg) {
                        Ok(Request::LobbyUnsubscribe) => {
                            srv.lobby.send(lobby::Command::Unsubscribe(id.clone())).await
                               .map_err(|x| error!("Lobby is unreachable: {}", x))?;
                        }
                        Ok(x) => {
                            warn!("Request not valid while in lobby: {:?}", x);
                            break;
                        }
                        Err(x) => {
                            warn!("Invalid request: {}", x);
                            break;
                        }
                    };
                }
                msg = rx.recv() => { match msg {
                    Some(lobby::Event::Subscribed(seed)) => send!(wsout, Reply::LobbySubscribed { seed }),
                    Some(lobby::Event::New(info)) => send!(wsout, Reply::LobbyNew { info }),
                    Some(lobby::Event::Update(info)) => send!(wsout, Reply::LobbyUpdate { info }),
                    Some(lobby::Event::Delete(id)) => send!(wsout, Reply::LobbyDelete { id }),
                    Some(lobby::Event::Unsubscribed) => {
                        send!(wsout, Reply::LobbyUnsubscribed);
                        return Ok(());
                    }
                    None => {
                        error!("Lobby is unreachable");
                        break;
                    }
                }}
            };
        }
        Err(())
    }

    async fn stop(mut self) {
        match self.ws.close(None).await {
            Ok(()) | Err(TsError::ConnectionClosed) => info!("Client disconnected"),
            Err(x) => warn!("Could not close connection gracefully: {}", x),
        }
    }
}

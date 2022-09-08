use crate::master::Services;
use crate::proto::{self, Reply, Request};
use crate::tuning::{CHUNK_SIZE, PING_TIMEOUT, PIPE_BUFFER, QUEUE_BUFFER};
use crate::{game, lobby};
use futures_util::sink::Sink;
use futures_util::stream::Stream;
use futures_util::{SinkExt, StreamExt};
use std::fmt::Display;
use tokio::io::{split, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, DuplexStream};
use tokio::select;
use tokio::sync::broadcast::error::RecvError as BrRecvError;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Error as TsError;
use tokio_tungstenite::WebSocketStream;
use tracing::{error, info, instrument, warn};

#[derive(Debug)]
#[allow(dead_code)]
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

macro_rules! send2 {
    ($out:expr, $reply:expr) => {{
        let msg = match Reply::forge(&$reply) {
            Ok(x) => x,
            Err(x) => {
                error!("Cannot forge reply: {}", x);
                return Err(());
            }
        };
        if let Err(x) = $out.send(Message::Text(msg)).await {
            warn!("Cannot send reply: {}", x);
            return Err(());
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

macro_rules! oneshot_reply2 {
    ($srv:expr, $cmd:expr) => {{
        let (tx, rx) = oneshot::channel();
        if let Err(_) = $srv.send($cmd(tx)).await {
            error!("Cannot forward request to {}", stringify!($cmd));
            return Err(());
        }
        match rx.await {
            Ok(x) => x,
            Err(x) => {
                error!("Cannot get reply from {}: {}", stringify!($cmd), x);
                return Err(());
            }
        }
    }};
    ($srv:expr, $cmd:expr, $($arg:tt)+) => {{
        let (tx, rx) = oneshot::channel();
        if let Err(_) = $srv.send($cmd(tx, $($arg)+)).await {
            error!("Cannot forward request to {}", stringify!($cmd));
            return Err(());
        }
        match rx.await {
            Ok(x) => x,
            Err(x) => {
                error!("Cannot get reply from {}: {}", stringify!($cmd), x);
                return Err(());
            }
        }
    }};
}

macro_rules! handle_ping {
    ($msg:expr, $wsout:expr) => {{
        match $msg {
            Ok(x) => x,
            Err(_) => match $wsout.send(Message::Ping(Vec::new())).await {
                Ok(()) => continue,
                Err(x) => {
                    error!("Cannot send ping: {}", x);
                    break;
                }
            },
        }
    }};
}

macro_rules! wsrecv {
    ($wsin:expr) => {{
        timeout(Duration::from_secs_f64(PING_TIMEOUT), $wsin.next())
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
        loop {
            let msg = wsrecv!(wsin).await;
            let msg = match handle_ping!(msg, wsout) {
                Some(x) => x,
                None => break,
            };
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
            let msg = match handle_ping!(wsrecv!(wsin).await, wsout) {
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
                Request::GameList {} => {
                    let games = oneshot_reply!(self.srv.game, game::Command::GetList);
                    send!(wsout, Reply::GameList { games });
                }
                Request::GameDescription { name } => {
                    let description =
                        oneshot_reply!(self.srv.game, game::Command::GetDescription, name);
                    send!(wsout, Reply::GameDescription { description });
                }
                Request::LobbyList {} => {
                    let info = oneshot_reply!(self.srv.lobby, lobby::Command::GetList);
                    send!(wsout, Reply::LobbyList { info });
                }
                Request::LobbySubscribe {} => {
                    if let Err(()) = Self::lobby(&mut wsin, &mut wsout, &self.srv).await {
                        break;
                    }
                }
                Request::LobbyJoinMatch { id, name, password } => {
                    if let Err(()) =
                        Self::join_match(&mut wsin, &mut wsout, &self.srv, id, name, password).await
                    {
                        break;
                    }
                }
                Request::SpectateJoin { id } => {
                    if let Err(()) =
                        Self::spectate_match(&mut wsin, &mut wsout, &self.srv, id).await
                    {
                        break;
                    }
                }
                Request::GameNew {
                    name,
                    game,
                    params,
                    args,
                    password,
                    verification,
                } => {
                    let id = oneshot_reply!(
                        self.srv.lobby,
                        lobby::Command::NewGame,
                        name,
                        game,
                        params,
                        args,
                        password,
                        verification
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

    async fn spectate_match<
        X: Sink<Message> + Unpin,
        Y: Stream<Item = Result<Message, TsError>> + Unpin,
    >(
        wsin: &mut Y,
        wsout: &mut X,
        srv: &Services,
        id: String,
    ) -> Result<(), ()>
    where
        <X as Sink<Message>>::Error: Display,
    {
        let (mut rx, seed) =
            match oneshot_reply2!(srv.lobby, lobby::Command::SpectateMatch, id.clone()) {
                Ok((rx, x, seed)) => {
                    send2!(wsout, Reply::SpectateJoined { info: Ok(x) });
                    (rx, seed)
                }
                Err(x) => {
                    send2!(wsout, Reply::SpectateJoined { info: Err(x) });
                    return Ok(());
                }
            };
        if let Some(seed) = seed {
            send2!(wsout, Reply::SpectateStarted {});
            let mut base = 0;
            while base < seed.len() {
                if let Err(x) = wsout
                    .send(Message::Binary(
                        seed[base..(base + CHUNK_SIZE).min(seed.len())].into(),
                    ))
                    .await
                {
                    warn!("Cannot send spectator seed: {}", x);
                    return Err(());
                }
                base += CHUNK_SIZE;
            }
            send2!(wsout, Reply::SpectateSynced {});
        }
        loop {
            select! {
                msg = wsrecv!(wsin) => {
                    let msg = match handle_ping!(msg, wsout) {
                        Some(Ok(Message::Text(x))) => x,
                        Some(_) => continue,
                        None => {
                            drop(rx);
                            srv.lobby.send(lobby::Command::RefreshGame(id)).await
                               .map_err(|_| error!("Lobby is unreachable"))?;
                            break;
                        }
                    };
                    match Request::parse(&msg) {
                        Ok(Request::SpectateLeave {}) => {
                            drop(rx);
                            srv.lobby.send(lobby::Command::RefreshGame(id)).await
                               .map_err(|_| error!("Lobby is unreachable"))?;
                            send!(wsout, Reply::SpectateLeaved {});
                            return Ok(());
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
                    Ok(lobby::MatchEvent::Update(info)) => send!(wsout, Reply::LobbyUpdate { info }),
                    Ok(lobby::MatchEvent::Expired) => {
                        send!(wsout, Reply::LobbyDelete { id });
                        return Ok(());
                    },
                    Ok(lobby::MatchEvent::Started(_)) => {
                        send!(wsout, Reply::SpectateStarted {});
                        send!(wsout, Reply::SpectateSynced {});
                    }
                    Ok(lobby::MatchEvent::SpectatorData(data)) => {
                        if let Err(x) = wsout.send(Message::Binary(data)).await {
                            warn!("Cannot send spectator data: {}", x);
                            break;
                        }
                    }
                    Ok(lobby::MatchEvent::Ended) => {
                        send!(wsout, Reply::SpectateEnded {});
                        return Ok(());
                    }
                    Err(BrRecvError::Closed) => {
                        error!("Lobby is unreachable");
                        break;
                    }
                    Err(BrRecvError::Lagged(x)) => {
                        warn!("Client is {} updates behind: dropping", x);
                        break;
                    }
                }}
            }
        }
        Err(())
    }

    async fn join_match<
        X: Sink<Message> + Unpin,
        Y: Stream<Item = Result<Message, TsError>> + Unpin,
    >(
        wsin: &mut Y,
        wsout: &mut X,
        srv: &Services,
        id: String,
        name: String,
        password: Option<String>,
    ) -> Result<(), ()>
    where
        <X as Sink<Message>>::Error: Display,
    {
        let (tx, mut rx) = mpsc::channel(QUEUE_BUFFER);
        match oneshot_reply2!(
            srv.lobby,
            lobby::Command::JoinMatch,
            id.clone(),
            name.clone(),
            password,
            tx
        ) {
            Ok(x) => send2!(wsout, Reply::LobbyJoinedMatch { info: Ok(x) }),
            Err(x) => {
                send2!(wsout, Reply::LobbyJoinedMatch { info: Err(x) });
                return Ok(());
            }
        };
        loop {
            select! {
                msg = wsrecv!(wsin) => {
                    let msg = match handle_ping!(msg, wsout) {
                        Some(Ok(Message::Text(x))) => x,
                        Some(_) => continue,
                        None => {
                            if let Err(x) = oneshot_reply2!(srv.lobby, lobby::Command::LeaveMatch, id, name) {
                                error!("Cannot leave match: {}", x);
                            }
                            break;
                        }
                    };
                    match Request::parse(&msg) {
                        Ok(Request::LobbyLeaveMatch {}) => {
                            if let Err(x) = oneshot_reply2!(srv.lobby, lobby::Command::LeaveMatch, id, name) {
                                error!("Cannot leave match: {}", x);
                                break;
                            }
                            send!(wsout, Reply::LobbyLeavedMatch {});
                            return Ok(());
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
                    Some(lobby::MatchEvent::Update(info)) => send!(wsout, Reply::LobbyUpdate { info }),
                    Some(lobby::MatchEvent::Started(Some(stream))) => {
                        send!(wsout, Reply::MatchStarted {});
                        return Self::play(wsin, wsout, stream, rx).await;
                    }
                    Some(lobby::MatchEvent::Expired) => {
                        send!(wsout, Reply::LobbyDelete { id });
                        return Ok(());
                    },
                    Some(_) => {
                        error!("Wrong message: {:?}", msg);
                        break;
                    }
                    None => {
                        error!("Lobby is unreachable");
                        break;
                    }
                }}
            }
        }
        Err(())
    }

    async fn play<X: Sink<Message> + Unpin, Y: Stream<Item = Result<Message, TsError>> + Unpin>(
        wsin: &mut Y,
        wsout: &mut X,
        stream: DuplexStream,
        mut rx: mpsc::Receiver<lobby::MatchEvent>,
    ) -> Result<(), ()>
    where
        <X as Sink<Message>>::Error: Display,
    {
        let (mut ppin, mut ppout) = split(stream);
        let mut buffer = [0; PIPE_BUFFER];
        loop {
            select! {
                msg = wsrecv!(wsin) => {
                    let msg = match handle_ping!(msg, wsout) {
                        Some(Ok(Message::Binary(x))) => x,
                        Some(Ok(Message::Text(_))) => {
                            warn!("Client sent command while playing, ignoring");
                            continue;
                        }
                        Some(_) => continue,
                        None => {
                            warn!("Lost connection while playing");
                            break;
                        }
                    };
                    if let Err(x) = ppout.write_all(&msg).await {
                        warn!("Cannot write to game manager: {}", x);
                        break;
                    }
                }
                msg = rx.recv() => { match msg {
                    Some(lobby::MatchEvent::Update(info)) => send!(wsout, Reply::LobbyUpdate { info }),
                    Some(lobby::MatchEvent::Ended) => {
                        send!(wsout, Reply::MatchEnded {});
                        return Ok(());
                    }
                    Some(_) => continue,
                    None => {
                        error!("Message channel ended prematurely");
                        break;
                    }
                }}
                msg = ppin.read(&mut buffer) => {
                    let size = match msg {
                        Ok(x) => x,
                        Err(x) => {
                            error!("Game manager pipe ended prematurely: {}", x);
                            break;
                        }
                    };
                    if let Err(x) = wsout.send(Message::Binary(buffer[..size].into())).await {
                        warn!("Lost connection while playing: {}", x);
                        break;
                    }
                }
            }
        }
        Err(())
    }

    async fn lobby<X: Sink<Message> + Unpin, Y: Stream<Item = Result<Message, TsError>> + Unpin>(
        wsin: &mut Y,
        wsout: &mut X,
        srv: &Services,
    ) -> Result<(), ()>
    where
        <X as Sink<Message>>::Error: Display,
    {
        let (mut rx, seed) = oneshot_reply2!(srv.lobby, lobby::Command::Subscribe);
        send2!(wsout, Reply::LobbySubscribed { seed });
        loop {
            select! {
                msg = wsrecv!(wsin) => {
                    let msg = match handle_ping!(msg, wsout) {
                        Some(Ok(Message::Text(x))) => x,
                        Some(_) => continue,
                        None => break,
                    };
                    match Request::parse(&msg) {
                        Ok(Request::LobbyUnsubscribe {}) => {
                            send!(wsout, Reply::LobbyUnsubscribed {});
                            return Ok(());
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
                    Ok(lobby::Event::New(info)) => send!(wsout, Reply::LobbyNew { info }),
                    Ok(lobby::Event::Update(info)) => send!(wsout, Reply::LobbyUpdate { info }),
                    Ok(lobby::Event::Delete(id)) => send!(wsout, Reply::LobbyDelete { id }),
                    Err(BrRecvError::Closed) => {
                        error!("Lobby is unreachable");
                        break;
                    }
                    Err(BrRecvError::Lagged(x)) => {
                        warn!("Client is {} updates behind: dropping", x);
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

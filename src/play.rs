use crate::game;
use crate::lobby;
use crate::proto::MatchInfo;
use crate::tuning::{PIPE_BUFFER, QUEUE_BUFFER};
use std::collections::{BTreeMap, HashMap};
use tokio::io::{duplex, split, AsyncReadExt, DuplexStream};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::{select, spawn};
use tracing::{error, instrument, warn};

#[derive(Debug)]
pub(crate) enum MatchEvent {
    Update(MatchInfo),
    Started(Option<DuplexStream>),
    SpectatorData(Vec<u8>),
}

impl Clone for MatchEvent {
    fn clone(&self) -> MatchEvent {
        match self {
            MatchEvent::Update(x) => MatchEvent::Update(x.clone()),
            MatchEvent::Started(_) => MatchEvent::Started(None),
            MatchEvent::SpectatorData(x) => MatchEvent::SpectatorData(x.clone()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Command {
    Subscribe(oneshot::Sender<(broadcast::Receiver<MatchEvent>, Vec<u8>)>),
    Stop,
}

#[instrument(name = "game", skip(instance, bots, players, spectators, lobby))]
pub(crate) async fn start(
    mut instance: Box<dyn game::Instance>,
    bots: Vec<Box<dyn game::Bot>>,
    players: BTreeMap<String, mpsc::Sender<MatchEvent>>,
    spectators: broadcast::Sender<MatchEvent>,
    lobby: mpsc::Sender<lobby::Command>,
    game: String,
    id: String,
) -> mpsc::Sender<Command> {
    let (tx, mut rx) = mpsc::channel(QUEUE_BUFFER);
    let mtx = tx.clone();
    spawn(async move {
        let mut streams = HashMap::new();
        for (name, tx) in players.iter() {
            let (ph, gh) = duplex(PIPE_BUFFER);
            streams.insert(name.clone(), gh);
            if let Err(_) = tx.send(MatchEvent::Started(Some(ph))).await {
                warn!("Player \"{}\" left before start", name);
            }
        }
        for (i, mut bot) in bots.into_iter().enumerate() {
            let (bh, gh) = duplex(PIPE_BUFFER);
            streams.insert(format!("ServerBot${}", i), gh);
            spawn(async move {
                bot.start(bh).await;
            });
        }
        drop(spectators.send(MatchEvent::Started(None)));
        let (msh, gsh) = duplex(PIPE_BUFFER);
        let mut spectate = split(msh).0;
        let mut history: Vec<u8> = Vec::new();
        let mut buffer = [0; PIPE_BUFFER];
        spawn(async move {
            instance.start(streams, split(gsh).1, mtx).await;
        });
        loop {
            select! {
                result = spectate.read(&mut buffer) => {
                    let size = match result {
                        Ok(0) => continue,
                        Ok(x) => x,
                        Err(x) => {
                            error!("Cannot read spectator buffer: {}", x);
                            continue;
                        }
                    };
                    history.extend_from_slice(&buffer[..size]);
                    drop(spectators.send(MatchEvent::SpectatorData(Vec::from(&buffer[..size]))));
                }
                cmd = rx.recv() => { match cmd {
                    Some(Command::Subscribe(tx)) => {
                        if let Err(_) = tx.send((spectators.subscribe(), history.clone())) {
                            error!("Subscription send failed");
                        }
                    }
                    Some(Command::Stop) | None => {
                        if let Err(_) = lobby.send(lobby::Command::DeleteGame(id)).await {
                            error!("Cannot send delete request");
                        }
                        break;
                    }
                }}
            }
        }
    });
    tx
}
use crate::game;
use crate::lobby;
use crate::proto::MatchInfo;
use crate::tuning::QUEUE_BUFFER;
use std::collections::BTreeMap;
use tokio::spawn;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::instrument;

#[derive(Debug, Clone)]
pub(crate) enum MatchEvent {
    Update(MatchInfo),
    // Started(todo!()),
}

#[derive(Debug)]
pub(crate) enum Command {
    Subscribe(oneshot::Sender<(broadcast::Receiver<MatchEvent>, Vec<u8>)>),
}

#[instrument(name = "game", skip(instance, players, spectators, lobby))]
pub(crate) async fn start(
    instance: Box<dyn game::Instance>,
    players: BTreeMap<String, mpsc::Sender<MatchEvent>>,
    spectators: broadcast::Sender<MatchEvent>,
    lobby: mpsc::Sender<lobby::Command>,
    game: String,
    id: String,
) -> mpsc::Sender<Command> {
    let (tx, mut rx) = mpsc::channel(QUEUE_BUFFER);
    spawn(async move {});
    tx
}

use crate::game;
use crate::proto::MatchInfo;
use crate::tuning::{QUEUE_BUFFER, RANDOM_ID_SIZE};
use data_encoding::BASE32_DNSSEC;
use rand::Rng;
use std::collections::BTreeMap;
use tokio::spawn;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, warn};

fn gen_random_id<T: Rng>(rng: &mut T) -> String {
    BASE32_DNSSEC.encode(&rng.gen::<[u8; RANDOM_ID_SIZE]>())
}

macro_rules! gen_unique_id {
    ($rng:expr, $map:expr) => {{
        let mut id = gen_random_id(&mut $rng);
        while $map.contains_key(&id) {
            id = gen_random_id(&mut $rng);
        }
        id
    }};
}

#[derive(Debug)]
pub(crate) enum Command {
    GetList(oneshot::Sender<Vec<MatchInfo>>),
    Subscribe(oneshot::Sender<String>, mpsc::Sender<Event>),
    Unsubscribe(String),
}

#[derive(Debug)]
pub(crate) enum Event {
    Subscribed(Vec<MatchInfo>),
    New(MatchInfo),
    Update(MatchInfo),
    Delete(String),
    Unsubscribed,
}

macro_rules! send {
    ($channel:expr, $data:expr) => {
        if let Err(_) = $channel.send($data) {
            warn!("Channel closed prematurely");
        }
    };
}

pub(crate) async fn start<T: 'static + Rng + Send>(
    mut rng: T,
    game: mpsc::Sender<game::Command>,
) -> mpsc::Sender<Command> {
    let (tx, mut rx) = mpsc::channel(QUEUE_BUFFER);
    spawn(async move {
        let mut matches: BTreeMap<String, MatchInfo> = BTreeMap::new();
        let mut listeners: BTreeMap<String, mpsc::Sender<Event>> = BTreeMap::new();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::GetList(tx) => {
                    send!(tx, matches.values().cloned().collect());
                }
                Command::Subscribe(tx, sender) => {
                    let id = gen_unique_id!(rng, listeners);
                    listeners.insert(id.clone(), sender);
                    send!(tx, id);
                }
                Command::Unsubscribe(id) => {
                    if listeners.remove(&id).is_none() {
                        error!("Trying to unsubscribe a non-existent key: {}", id);
                    }
                }
            };
        }
    });
    tx
}

use crate::game;
use crate::proto::MatchInfo;
use crate::tuning::*;
use data_encoding::BASE32_DNSSEC;
use rand::Rng;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::UNIX_EPOCH;
use tokio::spawn;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Instant;
use tracing::{error, info, warn};

fn encode(id: u64) -> String {
    BASE32_DNSSEC.encode(&u64::to_ne_bytes(id))
}

fn decode(s: &str) -> Result<u64, String> {
    match BASE32_DNSSEC.decode(s.as_bytes()) {
        Ok(x) if x.len() == 8 => Ok(u64::from_ne_bytes([
            x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7],
        ])),
        Ok(_) => Err(format!("Invalid ID: wrong length")),
        Err(x) => Err(format!("Invalid ID: {}", x)),
    }
}

macro_rules! decode {
    ($s:expr) => {{
        match decode($s) {
            Ok(x) => x,
            Err(x) => {
                warn!("{}", x);
                continue;
            }
        }
    }};
}

macro_rules! gen_unique_id {
    ($rng:expr, $map:expr) => {{
        let mut id = $rng.gen();
        while $map.contains_key(&id) {
            id = $rng.gen();
        }
        id
    }};
}

#[derive(Debug)]
pub(crate) enum Command {
    GetList(oneshot::Sender<Vec<MatchInfo>>),
    NewGame(
        oneshot::Sender<Result<String, String>>,
        String,
        String,
        game::Params,
        HashMap<String, String>,
        bool,
    ),
    Subscribe(oneshot::Sender<String>, mpsc::Sender<Event>),
    Unsubscribe(String),
}

#[derive(Debug, Clone)]
pub(crate) enum Event {
    Subscribed(Vec<MatchInfo>),
    New(MatchInfo),
    Update(MatchInfo),
    Delete(String),
    Unsubscribed,
}

#[derive(Debug)]
struct Match {
    info: MatchInfo,
    instance: Box<dyn game::Instance>,
    hidden: bool,
}

macro_rules! send {
    ($channel:expr, $data:expr) => {
        if let Err(_) = $channel.send($data) {
            warn!("Channel closed prematurely");
        }
    };
}

macro_rules! recv {
    ($channel:expr, $ret:expr, $cmd:expr, $($arg:tt)+) => {{
        let (tx, rx) = oneshot::channel();
        if let Err(_) = $channel.send($cmd(tx, $($arg)+)).await {
            error!("Cannot forward request to {}", stringify!($cmd));
            send!($ret, Err(format!("Internal server error")));
            continue;
        };
        match rx.await {
            Ok(x) => x,
            Err(x) => {
                error!("Cannot get reply from {}: {}", stringify!($cmd), x);
                send!($ret, Err(format!("Internal server error")));
                continue;
            }
        }
    }};
}

macro_rules! send_event {
    ($listeners:expr, $event:expr) => {{
        let mut tbr = BTreeSet::new();
        let event = $event;
        for (key, tx) in &$listeners {
            if let Err(_) = tx.send(event.clone()).await {
                tbr.insert(*key);
            }
        }
        for key in tbr {
            if $listeners.remove(&key).is_none() {
                error!("BTreeSet consistency error");
            }
        }
    }};
}

macro_rules! matches_info {
    ($matches:expr) => {{
        $matches
            .values()
            .filter(|x| !x.hidden)
            .map(|x| x.info.clone())
            .collect()
    }};
}

pub(crate) async fn start<T: 'static + Rng + Send>(
    mut rng: T,
    username_regex: Regex,
    gamename_regex: Regex,
    game: mpsc::Sender<game::Command>,
) -> mpsc::Sender<Command> {
    let (tx, mut rx) = mpsc::channel(QUEUE_BUFFER);
    spawn(async move {
        let time_sys_offset = match UNIX_EPOCH.elapsed() {
            Ok(x) => x.as_secs(),
            Err(x) => {
                error!("Cannot get system time: {}", x);
                0
            }
        };
        let time_base = Instant::now();
        let get_time = || time_base.elapsed().as_secs() + time_sys_offset;
        let mut matches: BTreeMap<u64, Match> = BTreeMap::new();
        let mut listeners: BTreeMap<u64, mpsc::Sender<Event>> = BTreeMap::new();
        let mut reaper: BTreeSet<(u64, u64)> = BTreeSet::new();
        while let Some(cmd) = rx.recv().await {
            loop {
                let mut must_remove = false;
                let (time, id) = match reaper.iter().nth(0) {
                    Some((x, y)) => (*x, *y),
                    None => break,
                };
                if get_time() < time {
                    break;
                }
                if let Some(info) = matches.get(&id) {
                    must_remove = !info.info.running;
                }
                if must_remove {
                    let eid = encode(id);
                    info!("Reaping match {} for inactivity", eid);
                    match matches.remove(&id) {
                        Some(m) if !m.hidden => send_event!(listeners, Event::Delete(eid)),
                        Some(_) => {}
                        None => error!("BTreeMap consistency error"),
                    };
                }
                if !reaper.remove(&(time, id)) {
                    error!("BTreeSet consistency error");
                }
            }
            match cmd {
                Command::GetList(tx) => {
                    send!(tx, matches_info!(matches));
                }
                Command::Subscribe(tx, sender) => {
                    let id = gen_unique_id!(rng, listeners);
                    if let Err(x) = sender.send(Event::Subscribed(matches_info!(matches))).await {
                        error!("Subscriber channel closed prematurely: {}", x);
                        continue;
                    }
                    listeners.insert(id, sender);
                    send!(tx, encode(id));
                }
                Command::Unsubscribe(id) => {
                    let did = decode!(&id);
                    match listeners.remove(&did) {
                        Some(tx) => {
                            if let Err(x) = tx.send(Event::Unsubscribed).await {
                                error!("Unsubscriber channel closed prematurely: {}", x);
                            }
                        }
                        None => error!("Trying to unsubscribe a non-existent key: {}", id),
                    }
                }
                Command::NewGame(tx, name, gamename, params, args, hidden) => {
                    if matches.len() >= MAX_GAME_INSTANCES {
                        send!(tx, Err(format!("Server is at maximum capacity")));
                        continue;
                    }
                    if !gamename_regex.is_match(&name) {
                        send!(tx, Err(format!("\"{}\" is not a valid game name", name)));
                        continue;
                    }
                    let (instance, params) = match recv!(
                        game,
                        tx,
                        game::Command::NewGame,
                        gamename.clone(),
                        params,
                        args.clone()
                    ) {
                        Ok(x) => x,
                        Err(x) => {
                            send!(tx, Err(x));
                            continue;
                        }
                    };
                    let (players, timeout) = match (params.players, params.timeout) {
                        (Some(x), Some(y)) => (x, y),
                        _ => {
                            error!("Game \"{}\" gave empty parameters: {:?}", gamename, params);
                            send!(tx, Err(format!("Internal server error")));
                            continue;
                        }
                    };
                    if params.bots >= players {
                        send!(tx, Err(format!("Cannot have all server bots")));
                        continue;
                    }
                    if players > MAX_PLAYERS {
                        send!(
                            tx,
                            Err(format!("Too many players: {} > {}", players, MAX_PLAYERS))
                        );
                        continue;
                    }
                    if !(MIN_TIMEOUT <= timeout && timeout <= MAX_TIMEOUT) {
                        send!(
                            tx,
                            Err(format!(
                                "Timeout {} out of allowed range [{}; {}]",
                                timeout, MIN_TIMEOUT, MAX_TIMEOUT
                            ))
                        );
                        continue;
                    }
                    let id = gen_unique_id!(rng, matches);
                    let info = MatchInfo {
                        players: players,
                        bots: params.bots,
                        timeout: timeout,
                        args: args,
                        id: encode(id),
                        name: name,
                        game: gamename,
                        running: false,
                        time: get_time() + INSTANCE_LIFETIME,
                        connected: Vec::new(),
                        spectators: 0,
                    };
                    reaper.insert((info.time, id));
                    let data = Match {
                        info: info.clone(),
                        instance: instance,
                        hidden: hidden,
                    };
                    matches.insert(id, data);
                    if !hidden {
                        send_event!(listeners, Event::New(info));
                    }
                    send!(tx, Ok(encode(id)));
                }
            };
        }
    });
    tx
}

use crate::game;
use crate::play;
pub(crate) use crate::play::MatchEvent;
use crate::proto::MatchInfo;
use crate::tuning::*;
use data_encoding::BASE32_DNSSEC;
use rand::Rng;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::time::UNIX_EPOCH;
use tokio::spawn;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::{timeout_at, Duration, Instant};
use tracing::{debug, error, info, warn};

pub(crate) fn encode(id: u64) -> String {
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

macro_rules! decode2 {
    ($s:expr, $stream:expr) => {{
        match decode($s) {
            Ok(x) => x,
            Err(x) => {
                warn!("{}", x);
                if let Err(_) = $stream.send(Err(format!("{}", x))) {
                    warn!("Cannot send back error message");
                }
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
        Option<String>,
        Option<String>,
    ),
    Subscribe(oneshot::Sender<(broadcast::Receiver<Event>, Vec<MatchInfo>)>),
    JoinMatch(
        oneshot::Sender<Result<MatchInfo, String>>,
        String,
        String,
        Option<String>,
        mpsc::Sender<MatchEvent>,
    ),
    LeaveMatch(oneshot::Sender<Result<(), String>>, String, String),
    SpectateMatch(
        oneshot::Sender<
            Result<(broadcast::Receiver<MatchEvent>, MatchInfo, Option<Vec<u8>>), String>,
        >,
        String,
    ),
    RefreshGame(String),
    DeleteGame(String),
}

#[derive(Debug, Clone)]
pub(crate) enum Event {
    New(MatchInfo),
    Update(MatchInfo),
    Delete(String),
}

#[derive(Debug)]
struct Match {
    info: MatchInfo,
    instance: Option<Box<dyn game::Instance>>,
    password: Option<String>,
    expiration: Instant,
    players: BTreeMap<String, mpsc::Sender<MatchEvent>>,
    spectators: broadcast::Sender<MatchEvent>,
    play: Option<mpsc::Sender<play::Command>>,
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
    ($sender:expr, $event:expr) => {{
        match $sender.send($event).unwrap_or(0) {
            1 => debug!("Sent event to 1 listener"),
            n => debug!("Sent event to {} listeners", n),
        };
    }};
}

macro_rules! match_update {
    ($m:expr) => {{
        let mut changed = true;
        while changed {
            changed = false;
            let mut to_remove = Vec::new();
            $m.info.spectators = $m.spectators.receiver_count();
            for (id, tx) in $m.players.iter() {
                if let Err(_) = tx.send(MatchEvent::Update($m.info.clone())).await {
                    to_remove.push(id.clone());
                }
            }
            drop($m.spectators.send(MatchEvent::Update($m.info.clone())));
            if to_remove.len() > 0 {
                changed = true;
                for id in to_remove {
                    $m.players.remove(&id);
                    $m.info.connected.remove(&id);
                }
            }
        }
    }};
}

macro_rules! match_expired {
    ($m:expr) => {{
        for (id, tx) in $m.players.iter() {
            if let Err(_) = tx.send(MatchEvent::Expired).await {
                warn!("Cannot send expiration notification to \"{}\"", id);
            }
        }
        drop($m.spectators.send(MatchEvent::Expired));
    }};
}

macro_rules! matches_info {
    ($matches:expr) => {{
        $matches.values().map(|x| x.info.clone()).collect()
    }};
}

pub(crate) async fn start<T: 'static + Rng + Send>(
    mut rng: T,
    username_regex: Regex,
    gamename_regex: Regex,
    password_regex: Regex,
    verification_pw: String,
    game: mpsc::Sender<game::Command>,
) -> mpsc::Sender<Command> {
    let (tx, mut rx) = mpsc::channel(QUEUE_BUFFER);
    let mtx = tx.clone();
    spawn(async move {
        let time_sys_offset = match UNIX_EPOCH.elapsed() {
            Ok(x) => x.as_secs(),
            Err(x) => {
                error!("Cannot get system time: {}", x);
                0
            }
        };
        let time_base = Instant::now();
        let get_unix_time = |t: Instant| (t - time_base).as_secs() + time_sys_offset;
        let mut matches: BTreeMap<u64, Match> = BTreeMap::new();
        let (event_tx, _) = broadcast::channel(BROADCAST_BUFFER);
        let mut reaper: BTreeSet<(Instant, u64)> = BTreeSet::new();
        loop {
            let cmd = if let Some((t, _)) = reaper.iter().nth(0) {
                timeout_at(*t, rx.recv()).await.ok()
            } else {
                Some(rx.recv().await)
            };
            loop {
                let mut must_remove = false;
                let (time, id) = match reaper.iter().nth(0) {
                    Some((x, y)) => (*x, *y),
                    None => break,
                };
                let now = Instant::now();
                if now < time {
                    break;
                }
                if let Some(info) = matches.get(&id) {
                    must_remove = !info.info.running && now >= info.expiration;
                }
                if must_remove {
                    let eid = encode(id);
                    info!("Reaping match for inactivity: {}", eid);
                    match matches.remove(&id) {
                        Some(m) => {
                            match_expired!(m);
                            send_event!(event_tx, Event::Delete(eid))
                        }
                        None => error!("BTreeMap consistency error"),
                    };
                }
                if !reaper.remove(&(time, id)) {
                    error!("BTreeSet consistency error");
                }
            }
            let cmd = match cmd {
                Some(Some(x)) => x,
                Some(None) => break,
                None => continue,
            };
            match cmd {
                Command::GetList(tx) => {
                    send!(tx, matches_info!(matches));
                }
                Command::Subscribe(tx) => {
                    send!(tx, (event_tx.subscribe(), matches_info!(matches)));
                }
                Command::DeleteGame(id) => {
                    let eid = decode!(&id);
                    if let Some(_) = matches.remove(&eid) {
                        send_event!(event_tx, Event::Delete(id));
                    } else {
                        error!("Trying to delete non-existent game \"{}\"", id);
                    }
                }
                Command::RefreshGame(id) => {
                    let eid = decode!(&id);
                    let m = match matches.get_mut(&eid) {
                        Some(x) => x,
                        None => {
                            error!("Trying to refresh non-existent game \"{}\"", id);
                            continue;
                        }
                    };
                    match_update!(m);
                    send_event!(event_tx, Event::Update(m.info.clone()));
                }
                Command::SpectateMatch(tx, id) => {
                    let eid = decode2!(&id, tx);
                    let m = match matches.get_mut(&eid) {
                        Some(x) => x,
                        None => {
                            send!(tx, Err(format!("Game \"{}\" does not exists", id)));
                            continue;
                        }
                    };
                    if m.info.running {
                        let p = match m.play {
                            Some(ref x) => x,
                            None => {
                                error!("Option consistency error at Command::SpectateMatch");
                                continue;
                            }
                        };
                        let (otx, orx) = oneshot::channel();
                        if let Err(_) = p.send(play::Command::Subscribe(otx)).await {
                            error!("play::Command::Subscribe send failed");
                            continue;
                        }
                        let (sub, seed) = match orx.await {
                            Ok(x) => x,
                            Err(_) => {
                                error!("play::Command::Subscribe recv failed");
                                continue;
                            }
                        };
                        send!(tx, Ok((sub, m.info.clone(), Some(seed))));
                    } else {
                        send!(tx, Ok((m.spectators.subscribe(), m.info.clone(), None)));
                    }
                    match_update!(m);
                    send_event!(event_tx, Event::Update(m.info.clone()));
                }
                Command::LeaveMatch(tx, id, name) => {
                    let eid = decode2!(&id, tx);
                    let m = match matches.get_mut(&eid) {
                        Some(x) if !x.players.contains_key(&name) => {
                            send!(tx, Err(format!("\"{}\" is not in this game", name)));
                            continue;
                        }
                        Some(x) if x.info.running => {
                            send!(tx, Err(format!("Game \"{}\" is already running", id)));
                            continue;
                        }
                        Some(x) => x,
                        None => {
                            send!(tx, Err(format!("Game \"{}\" does not exists", id)));
                            continue;
                        }
                    };
                    send!(tx, Ok(()));
                    let now = Instant::now();
                    m.info.connected.remove(&name);
                    m.players.remove(&name);
                    reaper.remove(&(m.expiration, eid));
                    m.expiration = now + Duration::from_secs_f64(INSTANCE_LIFETIME);
                    m.info.time = get_unix_time(m.expiration);
                    reaper.insert((m.expiration, eid));
                    match_update!(m);
                    send_event!(event_tx, Event::Update(m.info.clone()));
                }
                Command::JoinMatch(tx, id, name, password, ctx) => {
                    let eid = decode2!(&id, tx);
                    if !username_regex.is_match(&name) {
                        send!(tx, Err(format!("\"{}\" is not a valid username", name)));
                        continue;
                    }
                    let m = match matches.get_mut(&eid) {
                        Some(x) if x.info.running => {
                            send!(tx, Err(format!("Game \"{}\" is already running", id)));
                            continue;
                        }
                        Some(x) if x.players.contains_key(&name) => {
                            send!(tx, Err(format!("Username \"{}\" already taken", name)));
                            continue;
                        }
                        Some(x) if x.password != password => {
                            send!(tx, Err(format!("Wrong password")));
                            continue;
                        }
                        Some(x) => x,
                        None => {
                            send!(tx, Err(format!("Game \"{}\" does not exists", id)));
                            continue;
                        }
                    };
                    send!(tx, Ok(m.info.clone()));
                    let now = Instant::now();
                    m.info.connected.insert(name.clone());
                    m.players.insert(name, ctx);
                    reaper.remove(&(m.expiration, eid));
                    m.expiration = now + Duration::from_secs_f64(INSTANCE_LIFETIME);
                    m.info.time = get_unix_time(m.expiration);
                    reaper.insert((m.expiration, eid));
                    match_update!(m);
                    send_event!(event_tx, Event::Update(m.info.clone()));
                    if m.players.len() + m.info.bots == m.info.players {
                        let instance = match m.instance.take() {
                            Some(x) => x,
                            None => {
                                error!("Option consistency error");
                                continue;
                            }
                        };
                        let (otx, orx) = oneshot::channel();
                        if let Err(_) = game
                            .send(game::Command::GenBots(
                                otx,
                                m.info.game.clone(),
                                m.info.bots,
                            ))
                            .await
                        {
                            error!("Cannot send request to game::Command::GenBots");
                            continue;
                        }
                        let bots = match orx.await {
                            Ok(Ok(x)) => x,
                            Ok(Err(x)) => {
                                error!("Wrong reply from game::Command::GenBots: {}", x);
                                continue;
                            }
                            Err(_) => {
                                error!("Cannot get reply from game::Command::GenBots");
                                continue;
                            }
                        };
                        m.info.running = true;
                        m.info.time = get_unix_time(Instant::now());
                        m.play = Some(
                            play::start(
                                instance,
                                bots,
                                m.players.clone(),
                                m.spectators.clone(),
                                mtx.clone(),
                                m.info.game.clone(),
                                id,
                            )
                            .await,
                        );
                        match_update!(m);
                        send_event!(event_tx, Event::Update(m.info.clone()));
                    }
                }
                Command::NewGame(tx, name, gamename, params, args, password, verification) => {
                    if matches.len() >= MAX_GAME_INSTANCES {
                        send!(tx, Err(format!("Server is at maximum capacity")));
                        continue;
                    }
                    if !gamename_regex.is_match(&name) {
                        send!(tx, Err(format!("\"{}\" is not a valid game name", name)));
                        continue;
                    }
                    if let Some(ref pw) = password {
                        if !password_regex.is_match(&pw) {
                            send!(tx, Err(format!("\"{}\" is not a valid password", pw)));
                            continue;
                        }
                    }
                    let verified = match verification {
                        Some(x) if x == verification_pw => true,
                        Some(x) => {
                            send!(
                                tx,
                                Err(format!("\"{}\" is the wrong verification password", x))
                            );
                            continue;
                        }
                        None => false,
                    };
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
                    let expiry_time = Instant::now() + Duration::from_secs_f64(INSTANCE_LIFETIME);
                    let info = MatchInfo {
                        players: players,
                        bots: params.bots,
                        timeout: timeout,
                        args: args,
                        id: encode(id),
                        name: name,
                        game: gamename,
                        running: false,
                        time: get_unix_time(expiry_time),
                        connected: HashSet::new(),
                        spectators: 0,
                        password: password.is_some(),
                        verified: verified,
                    };
                    reaper.insert((expiry_time, id));
                    let data = Match {
                        info: info.clone(),
                        instance: Some(instance),
                        expiration: expiry_time,
                        players: BTreeMap::new(),
                        password: password,
                        spectators: broadcast::channel(BROADCAST_BUFFER).0,
                        play: None,
                    };
                    info!("Game of \"{}\" created: {}", data.info.game, encode(id));
                    matches.insert(id, data);
                    send_event!(event_tx, Event::New(info));
                    send!(tx, Ok(encode(id)));
                }
            };
        }
    });
    tx
}

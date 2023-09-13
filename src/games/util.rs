use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::io::{split, BufReader, DuplexStream, ReadHalf, WriteHalf};

pub(crate) struct Player {
    pub(crate) name: String,
    pub(crate) input: BufReader<ReadHalf<DuplexStream>>,
    pub(crate) output: WriteHalf<DuplexStream>,
}

impl Player {
    pub(crate) fn from<T: Rng>(players: HashMap<String, DuplexStream>, rng: &mut T) -> Vec<Player> {
        let mut p = Vec::new();
        for (name, stream) in players.into_iter() {
            let (r, w) = split(stream);
            p.push(Player {
                name: name,
                input: BufReader::new(r),
                output: w,
            });
        }
        p.shuffle(rng);
        p
    }
}

pub(crate) fn arg<T: FromStr>(
    m: &HashMap<String, String>,
    a: &str,
    d: T,
) -> Result<T, <T as FromStr>::Err> {
    match m.get(a) {
        Some(x) => x.parse(),
        None => Ok(d),
    }
}

macro_rules! lnout {
    ($stream:expr, $msg:expr) => {{
        let msg = String::from($msg) + "\n";
        match $stream.write_all(msg.as_bytes()).await {
            Ok(_) => {}
            Err(x) => {
                error!("Cannot write to stream: {}", x);
                return;
            }
        }
    }};
}

macro_rules! lnin {
    ($stream:expr) => {{
        let mut s = String::new();
        match $stream.read_line(&mut s).await {
            Ok(_) => s.trim().to_string(),
            Err(x) => {
                error!("Cannot read from stream: {}", x);
                return;
            }
        }
    }};
}

macro_rules! lnout2 {
    ($stream:expr, $msg:expr) => {{
        let msg = String::from($msg) + "\n";
        match $stream.write_all(msg.as_bytes()).await {
            Ok(_) => true,
            Err(x) => {
                warn!("Cannot write to stream: {}", x);
                false
            }
        }
    }};
}


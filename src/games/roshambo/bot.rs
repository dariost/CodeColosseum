use crate::game;
use async_trait::async_trait;
use rand::rngs::{OsRng, StdRng};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};
use tracing::error;

#[derive(Debug)]
pub(crate) struct Bot {}

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

#[async_trait]
impl game::Bot for Bot {
    async fn start(&mut self, stream: DuplexStream) {
        const MOVES: [&str; 3] = ["ROCK", "PAPER", "SCISSORS"];
        let mut rng = StdRng::from_rng(OsRng).expect("Cannot initialize PRNG");
        let (input, mut output) = split(stream);
        let mut input = BufReader::new(input);
        lnin!(input); // Read my name
        lnin!(input); // Read opponent name
        let rounds: usize = lnin!(input).parse().expect("Cannot parse number of rounds");
        for _ in 0..rounds {
            lnout!(output, *MOVES.choose(&mut rng).expect("Cannot fail"));
            if lnin!(input) == "RETIRE" {
                break;
            }
        }
    }
}

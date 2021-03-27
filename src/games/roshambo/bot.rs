use crate::game;
use async_trait::async_trait;
use rand::rngs::{OsRng, StdRng};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};
use tracing::error;

#[derive(Debug)]
pub(crate) struct Bot {}

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

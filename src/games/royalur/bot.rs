use super::logic::Board;
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
        let mut board = Board::new();
        let mut rng = StdRng::from_rng(OsRng).expect("Cannot initialize PRNG");
        let (input, mut output) = split(stream);
        let mut input = BufReader::new(input);
        lnin!(input); // Read my name
        lnin!(input); // Read opponent name
        let me: usize = lnin!(input).parse().expect("Cannot parse player number");
        let mut turn = 0;
        while !board.finished() {
            let roll: usize = lnin!(input)
                .trim()
                .split(" ")
                .map(|x| x.parse::<usize>().expect("Cannot parse die roll"))
                .sum();
            if turn == me {
                if let Some(x) = board.valid_moves(me, roll).choose(&mut rng) {
                    if board.make_move(me, *x, roll).expect("Cannot fail") {
                        turn = 1 - turn;
                    }
                    lnout!(output, format!("{}", x));
                }
            } else {
                if board.valid_moves(1 - me, roll).len() > 0 {
                    let token: usize = match lnin!(input).as_str() {
                        "RETIRE" => break,
                        x => x.parse().expect("Server sent garbage token"),
                    };
                    if board
                        .make_move(1 - me, token, roll)
                        .expect("Server sent invalid move")
                    {
                        turn = 1 - turn;
                    }
                }
            }
            turn = 1 - turn;
        }
    }
}

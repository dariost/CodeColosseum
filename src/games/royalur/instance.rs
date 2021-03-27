use super::super::util::Player;
use super::logic::Board;
use crate::game;
use async_trait::async_trait;
use rand::rngs::StdRng;
use rand::Rng;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, DuplexStream, WriteHalf};
use tokio::time::{sleep_until, timeout, Duration, Instant};
use tracing::warn;

#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) timeout: Duration,
    pub(crate) pace: Duration,
    pub(crate) rng: StdRng,
}

macro_rules! retired {
    ($other:expr, $spectators:expr) => {{
        lnout2!($other, "RETIRE");
        lnout2!($spectators, "RETIRE");
        break;
    }};
}

#[async_trait]
impl game::Instance for Instance {
    async fn start(
        &mut self,
        players: HashMap<String, DuplexStream>,
        mut spectators: WriteHalf<DuplexStream>,
    ) {
        let mut board = Board::new();
        let mut p = Player::from(players, &mut self.rng);
        assert_eq!(p.len(), 2);
        // Send names in order
        for i in 0..2 {
            lnout2!(p[0].output, &p[i].name);
            lnout2!(p[1].output, &p[i].name);
            lnout2!(spectators, &p[i].name);
        }
        // Send player index to players
        lnout2!(p[0].output, "0");
        lnout2!(p[1].output, "1");
        let mut turn = 0;
        while !board.finished() {
            let start = Instant::now();
            // Generate dice roll
            let d: Vec<_> = (0..4).map(|_| self.rng.gen::<bool>() as usize).collect();
            let roll = d.iter().sum::<usize>();
            let d: Vec<_> = d.into_iter().map(|x| format!("{}", x)).collect();
            let d = d.join(" ");
            // Send dice roll
            lnout2!(p[0].output, &d);
            lnout2!(p[1].output, &d);
            lnout2!(spectators, &d);
            // Read move
            let mut buffer = String::new();
            let token = match timeout(self.timeout, p[turn].input.read_line(&mut buffer)).await {
                // Timed out or closed connection
                Err(_) | Ok(Err(_)) => retired!(p[1 - turn].output, spectators),
                // Parse response
                Ok(Ok(_)) => match buffer.trim().parse::<usize>() {
                    // Token value too high
                    Ok(x) if x >= 7 => retired!(p[1 - turn].output, spectators),
                    // Valid token value
                    Ok(x) => x,
                    // Other garbage
                    Err(_) => retired!(p[1 - turn].output, spectators),
                },
            };
            // Keep the pace
            sleep_until(start + self.pace).await;
            let start = Instant::now();
            match board.make_move(turn, token, roll) {
                // Normal move
                Ok(again) => {
                    let m = format!("{}", token);
                    lnout2!(p[1 - turn].output, &m);
                    lnout2!(spectators, &m);
                    if again {
                        turn = 1 - turn;
                    }
                }
                // Wrong move
                Err(_) => retired!(p[1 - turn].output, spectators),
            }
            // Give turn to other player
            turn = 1 - turn;
            // Keep the pace
            sleep_until(start + self.pace).await;
        }
    }
}

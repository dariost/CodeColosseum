use super::super::util::Player;
use crate::game;
use async_trait::async_trait;
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, DuplexStream, WriteHalf};
use tokio::join;
use tokio::time::{sleep_until, timeout, Instant};
use tracing::warn;

#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) rounds: usize,
    pub(crate) timeout: f64,
    pub(crate) pace: f64,
    pub(crate) rng: StdRng,
}

macro_rules! process {
    ($result:expr, $line:expr) => {{
        match $result {
            Err(_) | Ok(Err(_)) => None,
            Ok(Ok(_)) => match $line.trim() {
                "ROCK" => Some("ROCK"),
                "PAPER" => Some("PAPER"),
                "SCISSORS" => Some("SCISSORS"),
                _ => None,
            },
        }
    }};
}

#[async_trait]
impl game::Instance for Instance {
    async fn start(
        &mut self,
        players: HashMap<String, DuplexStream>,
        mut spectators: WriteHalf<DuplexStream>,
    ) {
        let mut p = Player::from(players, &mut self.rng);
        assert_eq!(p.len(), 2);
        for i in 0..2 {
            lnout2!(spectators, &p[i].name);
            lnout2!(p[i].output, &p[i].name);
            lnout2!(p[i].output, &p[1 - i].name);
            lnout2!(p[i].output, format!("{}", self.rounds));
        }
        lnout2!(spectators, format!("{}", self.rounds));
        let tout = Duration::from_secs_f64(self.timeout);
        let pace = Duration::from_secs_f64(self.pace);
        let mut p1 = p.pop().expect("Cannot fail");
        let mut p0 = p.pop().expect("Cannot fail");
        for _ in 0..self.rounds {
            let start = Instant::now();
            let mut l0 = String::new();
            let mut l1 = String::new();
            let (r0, r1) = join!(
                timeout(tout, p0.input.read_line(&mut l0)),
                timeout(tout, p1.input.read_line(&mut l1))
            );
            let r0 = process!(r0, l0);
            let r1 = process!(r1, l1);
            sleep_until(start + pace).await;
            match (r0, r1) {
                (Some(x), Some(y)) => {
                    lnout2!(p1.output, x);
                    lnout2!(spectators, x);
                    lnout2!(p0.output, y);
                    lnout2!(spectators, y);
                }
                (None, Some(y)) => {
                    lnout2!(p1.output, "RETIRE");
                    lnout2!(spectators, "RETIRE");
                    lnout2!(spectators, y);
                    break;
                }
                (Some(x), None) => {
                    lnout2!(p0.output, "RETIRE");
                    lnout2!(spectators, x);
                    lnout2!(spectators, "RETIRE");
                    break;
                }
                (None, None) => {
                    lnout2!(spectators, "RETIRE");
                    lnout2!(spectators, "RETIRE");
                    break;
                }
            };
        }
    }

    async fn args(&self) -> HashMap<String, String> {
        HashMap::from([
            ("pace".to_owned(), format!("{:?}", self.pace)),
            ("rounds".to_owned(), format!("{}", self.rounds))
        ])
    }
}

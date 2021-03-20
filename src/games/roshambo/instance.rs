use crate::game;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{
    split, AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream, ReadHalf, WriteHalf,
};
use tokio::join;
use tokio::time::{sleep_until, timeout, Instant};
use tracing::warn;

#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) rounds: usize,
    pub(crate) timeout: f64,
    pub(crate) pace: f64,
}

struct Player {
    name: String,
    input: BufReader<ReadHalf<DuplexStream>>,
    output: WriteHalf<DuplexStream>,
}

macro_rules! lnout {
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
        let mut p = Vec::new();
        for (name, stream) in players.into_iter() {
            let (r, w) = split(stream);
            lnout!(spectators, &name);
            p.push(Player {
                name: name,
                input: BufReader::new(r),
                output: w,
            });
        }
        lnout!(spectators, format!("{}", self.rounds));
        assert_eq!(p.len(), 2);
        for i in 0..2 {
            lnout!(p[i].output, &p[i].name);
            lnout!(p[i].output, &p[1 - i].name);
            lnout!(p[i].output, format!("{}", self.rounds));
        }
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
                    lnout!(p1.output, x);
                    lnout!(spectators, x);
                    lnout!(p0.output, y);
                    lnout!(spectators, y);
                }
                (None, Some(y)) => {
                    lnout!(p1.output, "RETIRE");
                    lnout!(spectators, "RETIRE");
                    lnout!(spectators, y);
                    break;
                }
                (Some(x), None) => {
                    lnout!(p0.output, "RETIRE");
                    lnout!(spectators, x);
                    lnout!(spectators, "RETIRE");
                    break;
                }
                (None, None) => {
                    lnout!(spectators, "RETIRE");
                    lnout!(spectators, "RETIRE");
                    break;
                }
            };
        }
    }
}

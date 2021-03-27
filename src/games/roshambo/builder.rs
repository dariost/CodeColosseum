use super::bot::Bot;
use super::instance::Instance;
use crate::game;
use crate::games;
use async_trait::async_trait;
use games::util::arg;
use rand::rngs::{OsRng, StdRng};
use rand::SeedableRng;
use std::collections::HashMap;

const DEFAULT_TIMEOUT: f64 = 30.0;
const DEFAULT_ROUNDS: usize = 10;
const DEFAULT_PACE: f64 = 1.0;

#[derive(Debug)]
pub(crate) struct Builder {}

impl Builder {
    pub(crate) fn new() -> Box<dyn game::Builder> {
        Box::new(Builder {})
    }
}

#[async_trait]
impl game::Builder for Builder {
    fn name(&self) -> &str {
        "roshambo"
    }
    async fn description(&self) -> String {
        String::from(include_str!("description.md"))
    }
    async fn gen_instance(
        &self,
        param: &mut game::Params,
        args: HashMap<String, String>,
    ) -> Result<Box<dyn game::Instance>, String> {
        param.players = match param.players {
            Some(2) => Some(2),
            Some(x) => return Err(format!("Cannot create game with {} players", x)),
            None => Some(2),
        };
        param.timeout = param.timeout.or(Some(DEFAULT_TIMEOUT));
        let rounds = match arg(&args, "rounds", DEFAULT_ROUNDS) {
            Ok(0) => return Err(format!("Cannot play for 0 rounds")),
            Ok(x) if x > 10000 => return Err(format!("Too many rounds")),
            Ok(x) => x,
            Err(x) => return Err(format!("Invaid number of rounds: {}", x)),
        };
        let pace = match arg(&args, "pace", DEFAULT_PACE) {
            Ok(x) if x < 0.0 || x > 30.0 => return Err(format!("Invalid pace")),
            Ok(x) => x,
            Err(x) => return Err(format!("Invaid pace: {}", x)),
        };
        let rng = match StdRng::from_rng(OsRng) {
            Ok(x) => x,
            Err(x) => return Err(format!("Cannot initialize PRNG: {}", x)),
        };
        Ok(Box::new(Instance {
            rounds: rounds,
            timeout: param.timeout.expect("Cannot fail"),
            pace: pace,
            rng: rng,
        }))
    }
    async fn gen_bot(&self) -> Box<dyn game::Bot> {
        Box::new(Bot {})
    }
}

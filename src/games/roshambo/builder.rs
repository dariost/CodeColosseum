use super::bot::Bot;
use super::instance::Instance;
use crate::game;
use crate::games;
use crate::proto::GameArgInfo;
use async_trait::async_trait;
use games::util::arg;
use rand::rngs::{OsRng, StdRng};
use rand::SeedableRng;
use regex::Regex;
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
    async fn args(&self) -> HashMap<String, GameArgInfo> {
        HashMap::from([
            (
                "rounds".to_owned(),
                GameArgInfo {
                    description: "How many rounds (1-9999)".to_owned(),
                    regex: "^([1-9][0-9]{0,3})$".to_owned(),
                },
            ),
            (
                "pace".to_owned(),
                GameArgInfo {
                    description: "How fast the game plays (0-30)".to_owned(),
                    regex: "^(30|([12][0-9]|[0-9])(.[0-9]*)?)$".to_owned(),
                },
            ),
        ])
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
        let constraints = self.args().await;

        let rounds_reg = Regex::new(&constraints["rounds"].regex).unwrap();
        let rounds = match arg(&args, "rounds", DEFAULT_ROUNDS) {
            Ok(x) => {
                if !rounds_reg.is_match(&x.to_string()) {
                    return Err(format!("Invalid number of rounds"));
                } else {
                    x
                }
            }
            Err(x) => return Err(format!("Invaid number of rounds: {}", x)),
        };

        let pace_reg = Regex::new(&constraints["pace"].regex).unwrap();
        let pace = match arg(&args, "pace", DEFAULT_PACE) {
            Ok(x) => {
                if !pace_reg.is_match(&x.to_string()) {
                    return Err(format!("Invalid pace"));
                } else {
                    x
                }
            }
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

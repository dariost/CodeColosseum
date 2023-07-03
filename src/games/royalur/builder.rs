use super::bot::Bot;
use super::instance::Instance;
use crate::game;
use crate::games;
use crate::proto::GameArgInfo;
use async_trait::async_trait;
use games::util::arg;
use rand::rngs::{OsRng, StdRng};
use rand::SeedableRng;
use std::collections::HashMap;
use std::hash::Hash;
use tokio::time::Duration;

const DEFAULT_TIMEOUT: f64 = 90.0;
const DEFAULT_PACE: f64 = 1.5;

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
        "royalur"
    }
    async fn description(&self) -> String {
        String::from(include_str!("description.md"))
    }

    async fn args(&self) -> HashMap<String, GameArgInfo> {
        HashMap::from([(
            "pace".to_owned(),
            GameArgInfo {
                description: "How fast the game plays (?)".to_owned(),
                max: Some(30.0),
                min: Some(0.0)
            },
        )])
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

        let pace = match arg(&args, "pace", DEFAULT_PACE) {
            Ok(x) => {
                let limits = &constraints["pace"];
                if (x as f64) < limits.min.unwrap() || (x as f64) > limits.max.unwrap() {
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
            timeout: Duration::from_secs_f64(param.timeout.expect("Cannot fail")),
            pace: Duration::from_secs_f64(pace),
            rng: rng,
        }))
    }
    async fn gen_bot(&self) -> Box<dyn game::Bot> {
        Box::new(Bot {})
    }
}

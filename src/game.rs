use crate::games;
pub(crate) use crate::proto::{GameArgInfo, GameParams as Params, GameUsage};
use crate::tuning::QUEUE_BUFFER;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::io::{DuplexStream, WriteHalf};
use tokio::spawn;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, warn};

#[async_trait]
pub(crate) trait Builder: Send + Sync + Debug {
    fn name(&self) -> &str;
    async fn description(&self) -> String;
    async fn args(&self) -> HashMap<String, GameArgInfo>;
    async fn gen_instance(
        &self,
        param: &mut Params,
        args: HashMap<String, String>,
    ) -> Result<Box<dyn Instance>, String>;
    async fn gen_bot(&self) -> Box<dyn Bot>;
}

#[async_trait]
pub(crate) trait Instance: Send + Sync + Debug {
    async fn start(
        &mut self,
        players: HashMap<String, DuplexStream>,
        spectators: WriteHalf<DuplexStream>,
    );

    /// Get arguments of this instance
    async fn args(&self) -> HashMap<String, String>;
}

#[async_trait]
pub(crate) trait Bot: Send + Sync + Debug {
    async fn start(&mut self, stream: DuplexStream);
}

#[derive(Debug)]
pub(crate) enum Command {
    GetList(oneshot::Sender<Vec<GameUsage>>),
    GetDescription(oneshot::Sender<Option<String>>, String),
    NewGame(
        oneshot::Sender<Result<(Box<dyn Instance>, Params), String>>,
        String,
        Params,
        HashMap<String, String>,
    ),
    GenBots(
        oneshot::Sender<Result<Vec<Box<dyn Bot>>, String>>,
        String,
        usize,
    ),
}

macro_rules! send {
    ($channel:expr, $data:expr) => {
        if let Err(_) = $channel.send($data) {
            warn!("Oneshot channel closed prematurely");
        }
    };
}

pub(crate) async fn start() -> mpsc::Sender<Command> {
    let (tx, mut rx) = mpsc::channel(QUEUE_BUFFER);
    spawn(async move {
        let mut games = HashMap::new();
        for game in games::get() {
            games.insert(String::from(game.name()), game);
        }
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::GetList(tx) => {
                    let mut result: Vec<GameUsage> = Vec::new();
                    for game in games::get() {
                        result.push(GameUsage {
                            name: game.name().to_owned(),
                            args: game.args().await,
                        });
                    }

                    send!(tx, result);
                }
                Command::GetDescription(tx, name) => {
                    let result = if let Some(game) = games.get(&name) {
                        Some(game.description().await)
                    } else {
                        None
                    };
                    send!(tx, result);
                }
                Command::GenBots(tx, name, n_bots) => {
                    let result = if let Some(game) = games.remove(&name) {
                        match spawn(async move {
                            let mut bots = Vec::new();
                            for _ in 0..n_bots {
                                bots.push(game.gen_bot().await);
                            }
                            (bots, game)
                        })
                        .await
                        {
                            Ok((result, game)) => {
                                games.insert(name, game);
                                Ok(result)
                            }
                            Err(x) => {
                                error!(
                                    "Fatal error while generating bots for game \"{}\": {}",
                                    name, x
                                );
                                Err(format!("Internal server error"))
                            }
                        }
                    } else {
                        Err(format!("Game not found"))
                    };
                    send!(tx, result);
                }
                Command::NewGame(tx, name, mut params, args) => {
                    let result = if let Some(game) = games.remove(&name) {
                        match spawn(async move {
                            let result = match game.gen_instance(&mut params, args).await {
                                Ok(instance) => Ok((instance, params)),
                                Err(x) => Err(format!("Cannot create game: {}", x)),
                            };
                            (result, game)
                        })
                        .await
                        {
                            Ok((result, game)) => {
                                games.insert(name, game);
                                result
                            }
                            Err(x) => {
                                error!(
                                    "Fatal error while generating instance of game \"{}\": {}",
                                    name, x
                                );
                                Err(format!("Internal server error"))
                            }
                        }
                    } else {
                        Err(format!("Game not found"))
                    };
                    send!(tx, result);
                }
            }
        }
    });
    tx
}

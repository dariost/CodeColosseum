use crate::games;
pub(crate) use crate::proto::GameParams as Params;
use crate::tuning::QUEUE_BUFFER;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::spawn;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, warn};

#[async_trait]
pub(crate) trait Builder: Send + Sync + Debug {
    async fn name(&self) -> &str;
    async fn description(&self) -> &str;
    async fn gen_instance(
        &self,
        param: &mut Params,
        args: HashMap<String, String>,
    ) -> Result<Box<dyn Instance>, String>;
    async fn gen_bot(&self) -> Box<dyn Bot>;
}

#[async_trait]
pub(crate) trait Instance: Send + Sync + Debug {}

#[async_trait]
pub(crate) trait Bot: Send + Sync + Debug {}

#[derive(Debug)]
pub(crate) enum Command {
    GetList(oneshot::Sender<Vec<String>>),
    GetDescription(oneshot::Sender<Option<String>>, String),
    NewGame(
        oneshot::Sender<Result<(Box<dyn Instance>, Params), String>>,
        String,
        Params,
        HashMap<String, String>,
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
            games.insert(String::from(game.name().await), game);
        }
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::GetList(tx) => {
                    send!(tx, games.keys().map(|x| x.clone()).collect());
                }
                Command::GetDescription(tx, name) => {
                    let result = if let Some(game) = games.get(&name) {
                        Some(game.description().await.to_string())
                    } else {
                        None
                    };
                    send!(tx, result);
                }
                Command::NewGame(tx, name, mut params, args) => {
                    let result = if let Some(game) = games.remove(&name) {
                        match spawn(async move {
                            let result = match game.gen_instance(&mut params, args).await {
                                Ok(x) => Ok((x, params)),
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

use crate::games;
use crate::tuning::QUEUE_BUFFER;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::spawn;
use tokio::sync::{mpsc, oneshot};
use tracing::{instrument, warn};

#[async_trait]
pub(crate) trait Builder: Send + Sync {
    async fn name(&self) -> &str;
    async fn description(&self) -> &str;
    async fn gen_instance(
        &self,
        args: HashMap<String, String>,
    ) -> Result<Box<dyn Instance>, String>;
    async fn gen_bot(&self) -> Box<dyn Bot>;
}

#[async_trait]
pub(crate) trait Instance: Send + Sync {}

#[async_trait]
pub(crate) trait Bot: Send + Sync {}

pub(crate) enum Command {
    GetList(oneshot::Sender<Vec<String>>),
    GetDescription(oneshot::Sender<Option<String>>, String),
}

macro_rules! send {
    ($channel:expr, $data:expr) => {
        if let Err(_) = $channel.send($data) {
            warn!("Oneshot channel closed prematurely");
        }
    };
}

#[instrument(name = "games_manager")]
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
                    send!(
                        tx,
                        if let Some(game) = games.get(&name) {
                            Some(game.description().await.to_string())
                        } else {
                            None
                        }
                    );
                }
            }
        }
    });
    tx
}

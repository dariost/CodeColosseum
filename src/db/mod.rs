use std::path::PathBuf;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tokio::sync::{mpsc, oneshot};
use tracing::info;

use crate::tuning::QUEUE_BUFFER;

pub(crate) mod filesystem;

/// All the informations that we want to store about a match
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MatchData {
    pub id: String,
    pub game_name: String,
    pub args: HashMap<String, String>,
    pub players: Vec<String>,
    pub bot_count: u32,
    pub history: Vec<u8>,
}

impl fmt::Display for MatchData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "id: {}", self.id)?;
        writeln!(f, "game_name: {}", self.game_name)?;
        writeln!(f, "bot_count: {}", self.bot_count)?;

        writeln!(f, "args:")?;
        for (key, value) in self.args.iter() {
            writeln!(f, "- {}: {}", key, value)?;
        }

        writeln!(f, "players:")?;
        for player in &self.players {
            writeln!(f, "- {}", player)?;
        }

        match std::str::from_utf8(&self.history) {
            Err(_) => Err(fmt::Error),
            Ok(history) => writeln!(f, "\n{}", history),
        }
    }
}

/// All possible database errors
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum DatabaseError {
    FileNotFound,
    UnableToDeserialize,
    InvalidID,
}

#[derive(Debug)]
pub(crate) enum Command {
    /// Request the list of all saved matches ids
    List(oneshot::Sender<Vec<String>>),
    /// Returns the data of a match
    Retrieve {
        // TODO: Can be extendend to support queries
        response: oneshot::Sender<Result<MatchData, DatabaseError>>,
        id: String,
    },
    /// Save a match
    Store(MatchData),
}

#[async_trait]
pub(crate) trait Database {
    type Args: Send + 'static;

    /// Initialize the database
    fn create(args: Self::Args) -> Self;

    /// Close the database
    fn close(&mut self);

    /// Handle a command
    async fn execute(&mut self, cmd: Command);
}

#[derive(Clone, Debug)]
pub(crate) struct DatabaseHandle(mpsc::Sender<Command>);
impl std::ops::Deref for DatabaseHandle {
    type Target = mpsc::Sender<Command>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn start<D>(args: D::Args) -> DatabaseHandle
where
    D: Database + Send,
{
    let (tx, mut rx) = mpsc::channel(QUEUE_BUFFER);
    tokio::spawn(async move {
        let mut db = D::create(args);
        info!("Database created");

        loop {
            match rx.recv().await {
                Some(it) => db.execute(it).await,
                _ => break,
            }
        }

        db.close();
        info!("Database closed");
    });

    DatabaseHandle(tx)
}

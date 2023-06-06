use std::path::PathBuf;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tracing::info;

use crate::tuning::QUEUE_BUFFER;

pub(crate) mod filesystem;

/// All the informations that we want to store about a match
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MatchData {
    pub id: String,
    pub game_name: String,
    pub players: Vec<String>,
    pub history: Vec<u8>,
}

/// All possible database errors
#[derive(Debug)]
pub(crate) enum DatabaseError {
    FileNotFound,
    UnableToDeserialize,
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
pub(crate) struct DatabaseHandle {
    inner: mpsc::Sender<Command>,
}

impl std::ops::Deref for DatabaseHandle {
    type Target = mpsc::Sender<Command>;
    fn deref(&self) -> &Self::Target {
        &self.inner
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

    DatabaseHandle { inner: tx }
}

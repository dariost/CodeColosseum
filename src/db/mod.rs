use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use tracing::info;

use crate::tuning::QUEUE_BUFFER;

pub(crate) mod filesystem;

#[derive(Debug)]
pub(crate) enum Command {
    /// Request the list of all saved matches ids
    List(oneshot::Sender<Vec<String>>),
    /// Returns the history of a match
    Retrive {
        match_id: String,
        response: oneshot::Sender<Result<Vec<u8>, ()>>,
    },
    /// Save a match history
    Store {
        match_id: String,
        history: Vec<u8>,
        response: Option<oneshot::Sender<Result<(), ()>>>,
    },
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
        println!("Database created");

        loop {
            match rx.recv().await {
                Some(it) => db.execute(it).await,
                _ => break,
            }
        }

        db.close();
        println!("Database closed");
    });

    DatabaseHandle { inner: tx }
}

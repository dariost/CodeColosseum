use std::{ffi::OsString, path::PathBuf};

use crate::db::MatchData;

use super::{Command, Database, DatabaseError};
use async_trait::async_trait;
use tracing::error;

pub(crate) struct FileSystemArgs {
    pub(crate) root_dir: String,
}

pub(crate) struct FileSystem {
    args: FileSystemArgs,
}

#[async_trait]
impl Database for FileSystem {
    type Args = FileSystemArgs;

    fn create(args: Self::Args) -> Self {
        Self { args }
    }

    fn close(&mut self) {}

    async fn execute(&mut self, cmd: Command) {
        println!("Database handling cmd: {:?}", cmd);
        match cmd {
            // Return list of all saved games
            Command::List(response) => {
                let result: Vec<String> = match std::fs::read_dir(&self.args.root_dir) {
                    Ok(read_dir) => {
                        let paths: Vec<PathBuf> =
                            read_dir.filter_map(|e| e.ok()).map(|e| e.path()).collect();

                        paths
                            .iter()
                            .filter_map(|e| e.file_name())
                            .filter_map(|e| OsString::from(e).into_string().ok())
                            .collect()
                    }
                    Err(e) => {
                        error!("Unable to read database root dir: {}", e);
                        vec![]
                    }
                };

                if let Err(e) = response.send(result) {
                    error!("Unable to reply to list command: {:?}", e);
                }
            }
            // Save game data to file
            Command::Store(match_data) => {
                match serde_json::to_string_pretty(&match_data) {
                    Err(e) => error!("Unable to serialize game data: {}", e),
                    Ok(data_json) => {
                        // TODO: Solve path traversal vulnerability
                        let output_path = format!("{}/{}", self.args.root_dir, &match_data.id);
                        if let Err(e) = tokio::fs::write(output_path, data_json).await {
                            error!("Unable to save game data to file: {}", e);
                        }
                    }
                }
            }
            // Read game data from file
            Command::Retrieve { id, response } => {
                // TODO: Solve path traversal vulnerability
                let input_path = format!("{}/{}", self.args.root_dir, id);
                match tokio::fs::read(input_path).await {
                    Err(e) => {
                        error!("Unable to read game data: {}", e);
                        if let Err(e) = response.send(Err(DatabaseError::FileNotFound)) {
                            error!("Unable to send error: {:?}", e);
                        }
                    }
                    Ok(match_data_file) => {
                        match serde_json::from_slice::<MatchData>(&match_data_file) {
                            Err(e) => {
                                error!("Unable to deserialize game data: {}", e);
                                if let Err(e) =
                                    response.send(Err(DatabaseError::UnableToDeserialize))
                                {
                                    error!("Unable to send error: {:?}", e);
                                }
                            }
                            Ok(match_data) => {
                                if let Err(e) = response.send(Ok(match_data)) {
                                    error!("Unable to send history data: {:?}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

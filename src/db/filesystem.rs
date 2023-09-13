use std::{ffi::OsString, path::PathBuf};

use crate::db::MatchData;

use super::{Command, Database, DatabaseError};
use async_trait::async_trait;
use tracing::{error, info, trace};

const MATCH_DESCRIPTOR_FILE: &str = "descriptor.json";

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
        info!("Database handling cmd: {:?}", cmd);
        match cmd {
            // Return list of all saved games
            Command::List(response) => {
                let result: Vec<String> = match tokio::fs::read_dir(&self.args.root_dir).await {
                    Ok(mut read_dir) => {
                        let mut paths: Vec<PathBuf> = Vec::new();
                        loop {
                            match read_dir.next_entry().await {
                                Err(e) => error!("Unable to read directory entry: {}", e),
                                Ok(Some(file)) => paths.push(file.path()),
                                Ok(None) => break,
                            }
                        }

                        paths
                            .iter()
                            .filter(|e| e.is_dir())
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
            // Save game data to file and optionally call other system to process
            // match data
            Command::Store(match_data) => {
                // Only alphanumeric ids are allowed
                if !match_data.id.chars().all(char::is_alphanumeric) {
                    error!("Invalid game ID: {}", match_data.id);
                    return;
                }
                // Create directory for match data
                let match_directory = format!("{}/{}", self.args.root_dir, &match_data.id);
                if let Err(e) = tokio::fs::create_dir(&match_directory).await {
                    error!("Unable to create match directory: {}", e);
                    return;
                }

                // Serialize match data
                match serde_json::to_string_pretty(&match_data) {
                    Err(e) => error!("Unable to serialize game data: {}", e),
                    Ok(data_json) => {
                        // Write match descriptor
                        let output_path = format!("{}/{}", &match_directory, MATCH_DESCRIPTOR_FILE);
                        if let Err(e) = tokio::fs::write(output_path, data_json).await {
                            error!("Unable to save game data to file: {}", e);
                            return;
                        }
                    }
                }
            }
            // Read match descriptor from file
            Command::Retrieve { id, response } => {
                // Only alphanumeric ids are allowed
                if !id.chars().all(char::is_alphanumeric) {
                    error!("Invalid game ID: {}", id);
                    if let Err(e) = response.send(Err(DatabaseError::InvalidID)) {
                        error!("Unable to send error: {:?}", e);
                    }
                    return;
                }

                let input_path = format!("{}/{}/{}", self.args.root_dir, id, MATCH_DESCRIPTOR_FILE);
                trace!("reading match descriptor: {}", input_path);
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

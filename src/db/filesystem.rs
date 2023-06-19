use std::{ffi::OsString, path::PathBuf};

use crate::db::MatchData;

use super::{Command, Database, DatabaseError};
use async_trait::async_trait;
use tracing::{error, info};

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
                let result: Vec<String> = match std::fs::read_dir(&self.args.root_dir) {
                    Ok(read_dir) => {
                        let paths: Vec<PathBuf> =
                            read_dir.filter_map(|e| e.ok()).map(|e| e.path()).collect();

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
            // TODO: Solve path traversal vulnerability
            Command::Store(match_data) => {
                // Create directory for match data
                let match_directory = format!("{}/{}", self.args.root_dir, &match_data.id);
                if let Err(e) = std::fs::create_dir(&match_directory) {
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

                // Apply all post processes
                // TODO: Here you apply custom post process tools to generate batch resources
            }
            // Read match descriptor from ile
            // TODO: Solve path traversal vulnerability
            Command::Retrieve { id, response } => {
                let input_path = format!("{}/{}/{}", self.args.root_dir, id, MATCH_DESCRIPTOR_FILE);
                println!("{}", input_path);
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
            // Returns custom match resources
            // TODO: Solve path traversal vulnerability
            Command::Sync {
                response,
                id,
                target,
            } => {
                let input_path = format!("{}/{}/{}", self.args.root_dir, id, target);
                println!("reading custom resource of match {}: {}", id, target);
                match tokio::fs::read(input_path).await {
                    Err(e) => {
                        error!(
                            "Custom resource {} of match {} not found: {}",
                            id, target, e
                        );
                        if let Err(e) = response.send(Err(DatabaseError::SyncTargetNotFound)) {
                            error!("Unable to send error! {:?}", e);
                        }
                    }
                    Ok(input_data) => {
                        if let Err(e) = response.send(Ok(input_data)) {
                            error!("Unable to send error! {:?}", e);
                        }
                    }
                }
            }
        }
    }
}

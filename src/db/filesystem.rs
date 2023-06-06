use super::{Command, Database};
use async_trait::async_trait;
use tracing::info;

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
    }
}

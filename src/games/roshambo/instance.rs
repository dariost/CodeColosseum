use crate::{game, play};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::io::{DuplexStream, WriteHalf};
use tokio::sync::mpsc;

#[derive(Debug)]
pub(crate) struct Instance {}

#[async_trait]
impl game::Instance for Instance {
    async fn start(
        &mut self,
        players: HashMap<String, DuplexStream>,
        spectators: WriteHalf<DuplexStream>,
        control: mpsc::Sender<play::Command>,
    ) {
        todo!()
    }
}

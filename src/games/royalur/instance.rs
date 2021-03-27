use crate::game;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::io::{
    split, AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream, ReadHalf, WriteHalf,
};

#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) timeout: f64,
    pub(crate) pace: f64,
}

#[async_trait]
impl game::Instance for Instance {
    async fn start(
        &mut self,
        players: HashMap<String, DuplexStream>,
        mut spectators: WriteHalf<DuplexStream>,
    ) {
        todo!()
    }
}

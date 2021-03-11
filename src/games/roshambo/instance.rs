use crate::game;
use async_trait::async_trait;

#[derive(Debug)]
pub(crate) struct Instance {}

#[async_trait]
impl game::Instance for Instance {}

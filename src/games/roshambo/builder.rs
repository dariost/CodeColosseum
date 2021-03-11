use super::instance::Instance;
use crate::game;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Builder {}

impl Builder {
    pub(crate) fn new() -> Box<dyn game::Builder> {
        Box::new(Builder {})
    }
}

#[async_trait]
impl game::Builder for Builder {
    async fn name(&self) -> &str {
        "roshambo"
    }
    async fn description(&self) -> &str {
        include_str!("description.md")
    }
    async fn gen_instance(
        &self,
        param: &mut game::Params,
        _args: HashMap<String, String>,
    ) -> Result<Box<dyn game::Instance>, String> {
        param.players = Some(2);
        param.timeout = Some(1.0);
        Ok(Box::new(Instance {}))
    }
    async fn gen_bot(&self) -> Box<dyn game::Bot> {
        todo!()
    }
}

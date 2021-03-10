use crate::game;
use async_trait::async_trait;
use std::collections::HashMap;

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
        _param: &mut game::Params,
        _args: HashMap<String, String>,
    ) -> Result<Box<dyn game::Instance>, String> {
        todo!()
    }
    async fn gen_bot(&self) -> Box<dyn game::Bot> {
        todo!()
    }
}

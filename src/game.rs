use async_trait::async_trait;

#[async_trait]
trait Builder: Default {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

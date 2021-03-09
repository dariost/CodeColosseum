use serde::{Deserialize, Serialize};

pub(crate) const MAGIC: &str = "coco";
pub(crate) const VERSION: u64 = 1;

#[derive(Serialize, Deserialize)]
pub(crate) enum Request {
    Handshake(String, u64),
}

#[derive(Serialize, Deserialize)]
pub(crate) enum Reply {
    Handshake(String, u64),
}

impl Request {
    pub(crate) async fn forge(&self) -> Result<String, String> {
        match serde_json::to_string(self) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Cannot forge request: {}", x)),
        }
    }
    pub(crate) async fn parse(req: &str) -> Result<Request, String> {
        match serde_json::from_str(req) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Cannot parse request: {}", x)),
        }
    }
}

impl Reply {
    pub(crate) async fn forge(&self) -> Result<String, String> {
        match serde_json::to_string(self) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Cannot forge request: {}", x)),
        }
    }
    pub(crate) async fn parse(req: &str) -> Result<Reply, String> {
        match serde_json::from_str(req) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Cannot parse request: {}", x)),
        }
    }
}

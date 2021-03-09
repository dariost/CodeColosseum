use serde::{Deserialize, Serialize};

pub(crate) const MAGIC: &str = "coco";
pub(crate) const VERSION: u64 = 1;

#[derive(Serialize, Deserialize)]
pub(crate) enum Request {
    Handshake { magic: String, version: u64 },
    GameList,
    GameDescription { name: String },
}

#[derive(Serialize, Deserialize)]
pub(crate) enum Reply {
    Handshake { magic: String, version: u64 },
    GameList { games: Vec<String> },
    GameDescription { description: Option<String> },
}

#[allow(dead_code)]
impl Request {
    pub(crate) fn forge(&self) -> Result<String, String> {
        match serde_json::to_string(self) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Cannot forge request: {}", x)),
        }
    }
    pub(crate) fn parse(req: &str) -> Result<Request, String> {
        match serde_json::from_str(req) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Cannot parse request: {}", x)),
        }
    }
}

#[allow(dead_code)]
impl Reply {
    pub(crate) fn forge(&self) -> Result<String, String> {
        match serde_json::to_string(self) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Cannot forge request: {}", x)),
        }
    }
    pub(crate) fn parse(req: &str) -> Result<Reply, String> {
        match serde_json::from_str(req) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Cannot parse request: {}", x)),
        }
    }
}

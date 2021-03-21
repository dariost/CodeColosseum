pub(crate) const QUEUE_BUFFER: usize = 128;
pub(crate) const BROADCAST_BUFFER: usize = 512;
pub(crate) const PIPE_BUFFER: usize = 1 << 16;
pub(crate) const USERNAME_REGEX: &str = r"^[[:graph:]&&[^\$]]{1,16}$";
pub(crate) const PASSWORD_REGEX: &str = r"^[[:graph:]]{0,32}$";
pub(crate) const GAMENAME_REGEX: &str = r"^[[:print:]]{1,24}$";
pub(crate) const MAX_PLAYERS: usize = 100;
pub(crate) const MAX_GAME_INSTANCES: usize = 1000;
pub(crate) const MIN_TIMEOUT: f64 = 0.1;
pub(crate) const MAX_TIMEOUT: f64 = 600.0;
pub(crate) const INSTANCE_LIFETIME: f64 = 600.0;
pub(crate) const CHUNK_SIZE: usize = 1 << 20;
pub(crate) const END_GRACE_PERIOD: f64 = 0.25;

mod connection;
mod game;
mod games;
mod lobby;
mod master;
mod play;
mod proto;
mod tuning;

use clap::Clap;
use tokio::runtime::Runtime;
use tracing::error;

#[derive(Clap, Debug)]
struct CliArgs {
    #[clap(short, long, about = "Bind address", default_value = "127.0.0.1")]
    bind_address: String,
    #[clap(short('p'), long, about = "Listen port", default_value = "8088")]
    listen_port: u16,
    #[clap(short, long, about = "Verification password")]
    verification_password: Option<String>,
    #[cfg(unix)]
    #[clap(short, long, about = "Use bind address as a Unix Domain Socket")]
    unix_domain_socket: bool,
}

fn main() {
    tracing_subscriber::fmt::init();
    let args = CliArgs::parse();
    match Runtime::new() {
        Ok(rt) => rt.block_on(async move { master::start(args).await }),
        Err(x) => error!("Cannot create tokio runtime: {}", x),
    };
}

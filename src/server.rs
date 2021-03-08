mod game;
mod lobby;

use clap::Clap;
use tokio::runtime::Runtime;
use tracing::error;

#[derive(Clap, Debug)]
struct CliArgs {
    #[clap(short, long, about = "Bind address", default_value = "127.0.0.1")]
    bind_address: String,
    #[clap(short('p'), long, about = "Listen port", default_value = "8088")]
    listen_port: u16,
    #[cfg(unix)]
    #[clap(short, long, about = "Use bind address as a Unix Domain Socket")]
    unix_domain_socket: bool,
}

fn main() {
    tracing_subscriber::fmt::init();
    let opts = CliArgs::parse();
    match Runtime::new() {
        Ok(rt) => rt.block_on(async move { lobby::start(opts).await }),
        Err(x) => error!("cannot create tokio runtime -> {}", x),
    };
}

mod connection;
mod game;
mod games;
mod lobby;
mod master;
mod play;
mod proto;
mod tuning;

use clap::Parser;
use tokio::runtime::Runtime;
use tracing::error;
use tracing_subscriber::{
    filter::EnvFilter, layer::SubscriberExt, util::SubscriberInitExt, Registry,
};

#[derive(Parser, Debug)]
struct CliArgs {
    #[clap(short, long, help = "Bind address", default_value = "127.0.0.1")]
    bind_address: String,
    #[clap(short('p'), long, help = "Listen port", default_value = "8088")]
    listen_port: u16,
    #[clap(short, long, help = "Verification password")]
    verification_password: Option<String>,
    #[clap(short, long, help = "Send logs to journald")]
    journald: bool,
    #[cfg(unix)]
    #[clap(short, long, help = "Use bind address as a Unix Domain Socket")]
    unix_domain_socket: bool,
}

fn main() {
    let args = CliArgs::parse();
    if args.journald {
        let layer = match tracing_journald::layer() {
            Ok(x) => x,
            Err(x) => {
                println!("Cannot initialize journald logger: {}", x);
                return;
            }
        };
        Registry::default()
            .with(EnvFilter::from_default_env())
            .with(layer)
            .init();
    } else {
        tracing_subscriber::fmt::init();
    }
    match Runtime::new() {
        Ok(rt) => rt.block_on(async move { master::start(args).await }),
        Err(x) => error!("Cannot create tokio runtime: {}", x),
    };
}

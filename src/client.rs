mod proto;

use clap::Clap;
use regex::Regex;
use tokio::runtime::Runtime;
use tokio_tungstenite::connect_async;
use tracing::error;

#[derive(Clap, Debug)]
struct CliArgs {
    #[clap(
        short,
        long,
        about = "Server URL",
        default_value = "ws://127.0.0.1:8088/"
    )]
    server_url: String,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap, Debug)]
enum Command {
    #[clap(about = "List available games")]
    List(ListCommand),
    Lobby,
    New,
}

#[derive(Clap, Debug)]
struct ListCommand {
    #[clap(about = "Filter games with a regex")]
    filter: Option<Regex>,
    #[clap(short, long, about = "Show description even if no regex is provided")]
    verbose: bool,
}

fn init_logging() {
    if let Err(x) = tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .without_time()
                .with_target(false),
        )
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
    {
        println!("Cannot enable logging service: {}", x);
    }
}

async fn start(args: CliArgs) -> Result<(), String> {
    let ws = match connect_async(&args.server_url).await {
        Ok(x) => x,
        Err(x) => return Err(format!("Cannot connect to \"{}\": {}", args.server_url, x)),
    };
    match args.command {
        Command::List(cmd) => {}
        _ => todo!(),
    }
    Ok(())
}

fn main() {
    init_logging();
    let args = CliArgs::parse();
    match Runtime::new() {
        Ok(rt) => rt.block_on(async move {
            match start(args).await {
                Ok(()) => {}
                Err(x) => error!("{}", x),
            }
        }),
        Err(x) => error!("Cannot create tokio runtime: {}", x),
    };
}

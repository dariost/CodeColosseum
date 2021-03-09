mod proto;

use crate::proto::{Reply, Request};
use clap::Clap;
use futures_util::sink::Sink;
use futures_util::stream::Stream;
use futures_util::{SinkExt, StreamExt};
use std::fmt::Display;
use tokio::runtime::Runtime;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Error as TsError;
use tracing::{error, warn};

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

impl Command {
    async fn run<T: Sink<Message> + Unpin, U: Stream<Item = Result<Message, TsError>> + Unpin>(
        self,
        wsout: &mut T,
        wsin: &mut U,
    ) -> Result<(), String>
    where
        <T as Sink<Message>>::Error: Display,
    {
        match self {
            Command::List(cmd) => cmd.run(wsout, wsin).await,
            _ => todo!(),
        }
    }
}

#[derive(Clap, Debug)]
struct ListCommand {
    #[clap(about = "Show one game with its description")]
    filter: Option<String>,
}

impl ListCommand {
    async fn run<T: Sink<Message> + Unpin, U: Stream<Item = Result<Message, TsError>> + Unpin>(
        self,
        wsout: &mut T,
        wsin: &mut U,
    ) -> Result<(), String>
    where
        <T as Sink<Message>>::Error: Display,
    {
        let request = if let Some(name) = self.filter {
            Request::GameDescription { name }
        } else {
            Request::GameList
        };
        let request = match Request::forge(&request) {
            Ok(x) => Message::Text(x),
            Err(x) => return Err(format!("Cannot forge request: {}", x)),
        };
        if let Err(x) = wsout.send(request).await {
            return Err(format!("Cannot send request: {}", x));
        };
        loop {
            if let Some(msg) = wsin.next().await {
                match msg {
                    Ok(Message::Text(x)) => match Reply::parse(&x) {
                        Ok(Reply::GameList { games }) => {
                            for game in games {
                                println!("- {}", game);
                            }
                            break Ok(());
                        }
                        Ok(Reply::GameDescription { description: None }) => {
                            break Err(format!("Requested game could not be found"))
                        }
                        Ok(Reply::GameDescription {
                            description: Some(text),
                        }) => {
                            println!("{}", text);
                            break Ok(());
                        }
                        Ok(_) => return Err(format!("Server returned the wrong reply")),
                        Err(x) => return Err(format!("Could not parse server reply: {}", x)),
                    },
                    Err(x) => break Err(format!("Connection lost while waiting for reply: {}", x)),
                    Ok(_) => {}
                }
            } else {
                break Err(format!("Connection lost while waiting for reply"));
            }
        }
    }
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
    let mut ws = match connect_async(&args.server_url).await {
        Ok(x) => x.0,
        Err(x) => return Err(format!("Cannot connect to \"{}\": {}", args.server_url, x)),
    };
    let (mut wsout, mut wsin) = ws.split();
    let handshake_request = match Request::forge(&Request::Handshake {
        magic: proto::MAGIC.to_string(),
        version: proto::VERSION,
    }) {
        Ok(x) => Message::Text(x),
        Err(x) => return Err(format!("Cannot forge handshake request: {}", x)),
    };
    if let Err(x) = wsout.send(handshake_request).await {
        return Err(format!("Cannot send handshake request: {}", x));
    };
    let handshake_reply = loop {
        if let Some(msg) = wsin.next().await {
            match msg {
                Ok(Message::Text(x)) => match Reply::parse(&x) {
                    Ok(Reply::Handshake { magic, version }) => break (magic, version),
                    Ok(_) => return Err(format!("Server performed a wrong handshake")),
                    Err(x) => return Err(format!("Could not parse server handshake: {}", x)),
                },
                Err(x) => return Err(format!("Connection lost while performing handshake: {}", x)),
                Ok(_) => {}
            }
        } else {
            return Err(format!("Connection lost while performing handshake"));
        }
    };
    if !(handshake_reply.0 == proto::MAGIC && handshake_reply.1 == proto::VERSION) {
        return if handshake_reply.0 == proto::MAGIC {
            Err(format!(
                "Protocol version mismatch: local={}, server={}",
                proto::VERSION,
                handshake_reply.1
            ))
        } else {
            Err(format!(
                "\"{}\" is not a Code Colosseum server",
                args.server_url
            ))
        };
    }
    args.command.run(&mut wsout, &mut wsin).await?;
    ws = match wsin.reunite(wsout) {
        Ok(x) => x,
        Err(x) => return Err(format!("Cannot reunite streams {}", x)),
    };
    match ws.close(None).await {
        Ok(()) | Err(TsError::ConnectionClosed) => {}
        Err(x) => warn!("Could not close connection to server gracefully: {}", x),
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

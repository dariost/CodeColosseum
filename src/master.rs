use crate::connection::Client;
use crate::tuning::{GAMENAME_REGEX, USERNAME_REGEX};
use crate::{game, lobby};
use rand::rngs::{OsRng, StdRng};
use rand::SeedableRng;
use regex::Regex;
use std::error::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server as tuns;
use tracing::{error, info, warn};

#[cfg(unix)]
use {
    nix::{
        sys::stat::{umask, Mode},
        unistd::unlink,
    },
    tokio::net::UnixListener,
};

#[derive(Clone, Debug)]
pub(crate) struct Services {
    pub(crate) game: mpsc::Sender<game::Command>,
    pub(crate) lobby: mpsc::Sender<lobby::Command>,
}

async fn handle_raw_socket<T: AsyncRead + AsyncWrite + Unpin>(
    socket: T,
    mut address: String,
    services: Services,
) {
    let headers_callback = |request: &tuns::Request,
                            response: tuns::Response|
     -> Result<tuns::Response, tuns::ErrorResponse> {
        if let Some(x) = request.headers().get("x-forwarded-for") {
            if let Ok(x) = x.to_str() {
                address = x.to_string();
            }
        }
        Ok(response)
    };
    let websocket = match accept_hdr_async(socket, headers_callback).await {
        Ok(x) => x,
        Err(x) => {
            warn!(address = address.as_str(), "Cannot start connection: {}", x);
            return;
        }
    };
    let client = Client::new(websocket, address.clone(), services);
    client.start(&address).await;
}

async fn tcp_listener(args: crate::CliArgs, services: Services) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind((args.bind_address, args.listen_port)).await?;
    info!("Server up and running!");
    loop {
        let services = services.clone();
        let incoming_connection = listener.accept().await;
        spawn(async move {
            let (socket, addr) = match incoming_connection {
                Ok(x) => x,
                Err(x) => {
                    warn!("Connection refused: {}", x);
                    return;
                }
            };
            let address = format!("{}", addr);
            if let Err(x) = socket.set_nodelay(true) {
                warn!(address = address.as_str(), "Cannot set TCP_NODELAY: {}", x);
            }
            handle_raw_socket(socket, address, services).await;
        });
    }
}

#[cfg(unix)]
async fn uds_listener(args: crate::CliArgs, services: Services) -> Result<(), Box<dyn Error>> {
    umask(Mode::empty());
    drop(unlink(args.bind_address.as_str()));
    let listener = UnixListener::bind(args.bind_address)?;
    info!("Server up and running!");
    loop {
        let services = services.clone();
        let incoming_connection = listener.accept().await;
        spawn(async move {
            let (socket, addr) = match incoming_connection {
                Ok(x) => x,
                Err(x) => {
                    warn!("Connection refused: {}", x);
                    return;
                }
            };
            handle_raw_socket(socket, format!("{:?}", addr), services).await;
        });
    }
}

macro_rules! init {
    ($obj:expr) => {{
        match $obj {
            Ok(x) => x,
            Err(x) => {
                error!("Cannot initialize resource: {}", x);
                return;
            }
        }
    }};
}

pub(crate) async fn start(args: crate::CliArgs) {
    let rng = init!(StdRng::from_rng(OsRng));
    let username_regex = init!(Regex::new(USERNAME_REGEX));
    let gamename_regex = init!(Regex::new(GAMENAME_REGEX));
    let srv_game = game::start().await;
    let services = Services {
        game: srv_game.clone(),
        lobby: lobby::start(rng, username_regex, gamename_regex, srv_game).await,
    };
    #[cfg(unix)]
    let result = if args.unix_domain_socket {
        uds_listener(args, services).await
    } else {
        tcp_listener(args, services).await
    };
    #[cfg(not(unix))]
    let result = tcp_listener(opts, services).await;
    match result {
        Ok(()) => {}
        Err(x) => error!("Fatal error: {}", x),
    };
}

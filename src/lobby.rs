use std::error::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::spawn;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server as tuns;
use tokio_tungstenite::WebSocketStream;
use tracing::{error, info, warn};

#[cfg(unix)]
use {
    nix::{
        sys::stat::{umask, Mode},
        unistd::unlink,
    },
    tokio::net::UnixListener,
};

#[derive(Debug)]
pub(crate) struct Connection<T: AsyncRead + AsyncWrite + Unpin> {
    pub(crate) ws: WebSocketStream<T>,
    pub(crate) address: String,
}

impl<T: AsyncRead + AsyncWrite + Unpin> Connection<T> {
    fn new(ws: WebSocketStream<T>, address: String) -> Connection<T> {
        Connection { ws, address }
    }
}

async fn process_raw_socket<T: AsyncRead + AsyncWrite + Unpin>(socket: T, mut address: String) {
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
            warn!(
                address = address.as_str(),
                "cannot start connection -> {} |", x
            );
            return;
        }
    };
    info!(address = address.as_str(), "client connected |");
    let _ = Connection::new(websocket, address);
}

async fn tcp_listener(args: crate::CliArgs) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind((args.bind_address, args.listen_port)).await?;
    info!("server up and running!");
    loop {
        let incoming_connection = listener.accept().await;
        spawn(async move {
            let (socket, addr) = match incoming_connection {
                Ok(x) => x,
                Err(x) => {
                    warn!("connection refused -> {}", x);
                    return;
                }
            };
            process_raw_socket(socket, format!("{}", addr)).await;
        });
    }
}

#[cfg(unix)]
async fn uds_listener(args: crate::CliArgs) -> Result<(), Box<dyn Error>> {
    umask(Mode::empty());
    drop(unlink(args.bind_address.as_str()));
    let listener = UnixListener::bind(args.bind_address)?;
    info!("server up and running!");
    loop {
        let incoming_connection = listener.accept().await;
        spawn(async move {
            let (socket, addr) = match incoming_connection {
                Ok(x) => x,
                Err(x) => {
                    warn!("connection refused -> {}", x);
                    return;
                }
            };
            process_raw_socket(socket, format!("{:?}", addr)).await;
        });
    }
}

pub(crate) async fn start(args: crate::CliArgs) {
    #[cfg(unix)]
    let result = if args.unix_domain_socket {
        uds_listener(args).await
    } else {
        tcp_listener(args).await
    };
    #[cfg(not(unix))]
    let result = tcp_listener(opts).await;
    match result {
        Ok(()) => {}
        Err(x) => error!("fatal error -> {}", x),
    };
}

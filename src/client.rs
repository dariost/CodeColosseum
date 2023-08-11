mod db;
mod proto;
mod tuning;

use crate::proto::{GameParams, MatchInfo, Reply, Request};
use async_trait::async_trait;
use clap::{ArgEnum, Parser, Subcommand};
use futures_util::sink::Sink;
use futures_util::stream::Stream;
use futures_util::{SinkExt, StreamExt};
use prettytable::format::Alignment::CENTER;
use prettytable::{Attr, Cell, Row, Table};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{stdin, stdout, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::process as proc;
use tokio::runtime::Runtime;
use tokio::select;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Error as TsError;
use tokio_tungstenite::{connect_async, MaybeTlsStream};
use tracing::{error, warn};

#[cfg(unix)]
use {
    nix::{
        sys::stat::Mode,
        unistd::{mkfifo, Pid},
    },
    tempfile::tempdir,
    tokio::{fs::OpenOptions, join},
};

const BUFFER_SIZE: usize = 1 << 16;

#[derive(Parser, Debug)]
#[clap(version)]
struct CliArgs {
    #[clap(
        short,
        long,
        help = "Server URL",
        default_value = "ws://127.0.0.1:8088/"
    )]
    server_url: String,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// List available games
    List(ListCommand),
    /// Show games in lobby
    Lobby(LobbyCommand),
    /// Create new game
    New(NewCommand),
    /// Play or spectate a game
    Connect(ConnectCommand),
    /// List all saved matches
    History(HistoryCommand),
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
            Command::Lobby(cmd) => cmd.run(wsout, wsin).await,
            Command::New(cmd) => cmd.run(wsout, wsin).await,
            Command::Connect(cmd) => cmd.run(wsout, wsin).await,
            Command::History(cmd) => cmd.run(wsout, wsin).await,
        }
    }
}

async fn oneshot_request<
    T: Sink<Message> + Unpin,
    U: Stream<Item = Result<Message, TsError>> + Unpin,
>(
    request: Request,
    wsout: &mut T,
    wsin: &mut U,
) -> Result<Reply, String>
where
    <T as Sink<Message>>::Error: Display,
{
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
                    Ok(x) => break Ok(x),
                    Err(x) => break Err(format!("Could not parse server reply: {}", x)),
                },
                Err(x) => break Err(format!("Connection lost while waiting for reply: {}", x)),
                Ok(_) => {}
            }
        } else {
            break Err(format!("Connection lost while waiting for reply"));
        }
    }
}

#[derive(Parser, Debug)]
struct HistoryCommand {
    #[clap(help = "Show the match data of the game with this id")]
    id: Option<String>,
    #[clap(short, long, help = "Print the match data using json")]
    json: bool,
    #[clap(
        short,
        long,
        help = "Print only the history data as it appeared during play"
    )]
    direct: bool,
}

impl HistoryCommand {
    async fn run<T, U>(self, wsout: &mut T, wsin: &mut U) -> Result<(), String>
    where
        T: Sink<Message> + Unpin,
        <T as Sink<Message>>::Error: Display,
        U: Stream<Item = Result<Message, TsError>> + Unpin,
    {
        let request = match self.id {
            Some(id) => Request::HistoryMatch { id },
            None => Request::HistoryMatchList,
        };

        // Send request to server
        match oneshot_request(request, wsout, wsin).await? {
            Reply::HistoryMatch(match_data_result) => {
                match match_data_result {
                    Err(e) => println!("History error: {:?}", e),
                    Ok(match_data) => {
                        if self.direct {
                            match std::str::from_utf8(&match_data.history) {
                                Ok(match_history_string) => println!("{}", match_history_string),
                                Err(_) => return Err(format!("Unable to parse history data")),
                            }
                        } else {
                            if self.json {
                                let match_data_json =
                                    serde_json::to_string_pretty(&match_data).unwrap();
                                println!("{}", match_data_json);
                            } else {
                                println!("{:?}", match_data);
                            }
                        }
                    }
                }

                Ok(())
            }
            Reply::HistoryMatchList(matches) => {
                for value in &matches {
                    println!("{}", value);
                }
                Ok(())
            }
            _ => Err(format!("Server responded with wrong message")),
        }
    }
}

#[derive(Parser, Debug)]
struct ListCommand {
    #[clap(help = "Show one game with its description")]
    filter: Option<String>,
    #[clap(short, long, help = "Show games and their usage")]
    usage: bool,
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
            Request::GameList {}
        };
        match oneshot_request(request, wsout, wsin).await? {
            Reply::GameList { games } => {
                if self.usage {
                    let usage_text = serde_json::to_string_pretty(&games).unwrap();
                    println!("{}", usage_text);
                } else {
                    for game in games {
                        println!("- {}", game.name);
                    }
                }
                Ok(())
            }
            Reply::GameDescription { description: None } => {
                Err(format!("Requested game could not be found"))
            }
            Reply::GameDescription {
                description: Some(text),
            } => {
                println!("{}", text);
                Ok(())
            }
            _ => Err(format!("Server returned the wrong reply")),
        }
    }
}

#[derive(Parser, Debug)]
struct LobbyCommand {}

impl LobbyCommand {
    fn pretty_time_diff(a: &SystemTime, b: &SystemTime) -> String {
        if let Ok(x) = a.duration_since(*b) {
            let x = x.as_secs();
            if x >= 3600 {
                format!("{}h{}m{}s", x / 3600, (x % 3600) / 60, x % 60)
            } else if x >= 60 {
                format!("{}m{}s", x / 60, x % 60)
            } else {
                format!("{}s", x)
            }
        } else {
            String::from("???")
        }
    }
    async fn run<T: Sink<Message> + Unpin, U: Stream<Item = Result<Message, TsError>> + Unpin>(
        self,
        wsout: &mut T,
        wsin: &mut U,
    ) -> Result<(), String>
    where
        <T as Sink<Message>>::Error: Display,
    {
        let request = Request::LobbyList {};
        let info = match oneshot_request(request, wsout, wsin).await? {
            Reply::LobbyList { info } => info,
            _ => return Err(format!("Server returned the wrong reply")),
        };
        let mut table = Table::new();
        const FIELDS: &[&str] = &[
            "ID",
            "Verified",
            "Name",
            "Game",
            "Players",
            "Spectators",
            "Timeout",
            "Password",
            "Timing",
        ];
        table.add_row(Row::new(
            FIELDS
                .iter()
                .map(|x| Cell::new_align(x, CENTER).with_style(Attr::Bold))
                .collect(),
        ));
        let now = SystemTime::now();
        for game in info {
            let gametime = UNIX_EPOCH + Duration::from_secs(game.time);
            table.add_row(Row::new(vec![
                Cell::new_align(&game.id, CENTER),
                Cell::new_align(if game.verified { "X" } else { "" }, CENTER),
                Cell::new_align(&game.name, CENTER),
                Cell::new_align(&game.game, CENTER),
                Cell::new_align(
                    &format!("{}/{}", game.connected.len() + game.bots, game.players),
                    CENTER,
                ),
                Cell::new_align(&format!("{}", game.spectators), CENTER),
                Cell::new_align(&format!("{}", game.timeout), CENTER),
                Cell::new_align(if game.password { "X" } else { "" }, CENTER),
                Cell::new_align(
                    &if game.running {
                        format!("Running for {}", Self::pretty_time_diff(&now, &gametime))
                    } else {
                        format!("Expires in {}", Self::pretty_time_diff(&gametime, &now))
                    },
                    CENTER,
                ),
            ]));
        }
        table.printstd();
        Ok(())
    }
}

#[derive(Parser, Debug)]
struct NewCommand {
    #[clap(help = "Game to play")]
    game: String,
    #[clap(help = "Name for the lobby")]
    name: Option<String>,
    #[clap(short, long, help = "Password to join the game")]
    password: Option<String>,
    #[clap(short, long, help = "Password to create a verified game")]
    verification_password: Option<String>,
    #[clap(short('n'), long, help = "Number of players")]
    players: Option<usize>,
    #[clap(short, long, help = "Number of server bots", default_value = "0")]
    bots: usize,
    #[clap(short, long, help = "Timeout for player actions")]
    timeout: Option<f64>,
    #[clap(
        short,
        long("arg"),
        multiple = true,
        number_of_values = 1,
        help = "Additional arguments, can be specified multiple times with -a arg=val"
    )]
    args: Vec<String>,
}

impl NewCommand {
    async fn run<T: Sink<Message> + Unpin, U: Stream<Item = Result<Message, TsError>> + Unpin>(
        self,
        wsout: &mut T,
        wsin: &mut U,
    ) -> Result<(), String>
    where
        <T as Sink<Message>>::Error: Display,
    {
        let mut args = HashMap::new();
        for arg in self.args {
            let arg: Vec<_> = arg.split("=").collect();
            if arg.len() < 2 {
                return Err(format!("{} is not a valid argument", arg.join("")));
            }
            args.insert(arg[0].into(), arg[1..].join(""));
        }
        let name = self
            .name
            .unwrap_or_else(|| format!("{}'s game", whoami::username()));
        let request = Request::GameNew {
            game: self.game,
            name: name,
            params: GameParams {
                players: self.players,
                bots: self.bots,
                timeout: self.timeout,
            },
            args: args,
            password: self.password,
            verification: self.verification_password,
        };
        match oneshot_request(request, wsout, wsin).await? {
            Reply::GameNew { id: Ok(id) } => {
                println!("Created new game with id {}", id);
                Ok(())
            }
            Reply::GameNew { id: Err(x) } => {
                error!("Cannot create new game: {}", x);
                Ok(())
            }
            _ => Err(format!("Server returned the wrong reply")),
        }
    }
}

#[derive(Parser, Debug)]
struct ConnectCommand {
    #[clap(help = "Game ID")]
    id: String,
    #[clap(short, long, help = "Spectate instead of play")]
    spectate: bool,
    #[clap(short, long, help = "Username for the game")]
    name: Option<String>,
    #[clap(short, long, help = "Game password")]
    password: Option<String>,
    #[clap(
        arg_enum,
        short,
        long,
        help = "Channel for program communication",
        default_value = "stdio"
    )]
    channel: CommunicationChannel,
    #[clap(help = "Command to invoke", raw = true)]
    program: Vec<String>,
}

#[derive(ArgEnum, Debug, Clone)]
enum CommunicationChannel {
    Stdio,
    #[cfg(unix)]
    Pipe,
}

impl ConnectCommand {
    async fn run<T: Sink<Message> + Unpin, U: Stream<Item = Result<Message, TsError>> + Unpin>(
        self,
        wsout: &mut T,
        wsin: &mut U,
    ) -> Result<(), String>
    where
        <T as Sink<Message>>::Error: Display,
    {
        match self.channel {
            CommunicationChannel::Stdio => {
                if self.program.len() > 0 {
                    let mut prog = proc::Command::new(&self.program[0]);
                    if self.program.len() > 1 {
                        prog.args(&self.program[1..]);
                    }
                    if !self.spectate {
                        prog.stdout(Stdio::piped());
                    }
                    prog.stdin(Stdio::piped());
                    let mut prog = match prog.spawn() {
                        Ok(x) => x,
                        Err(x) => return Err(format!("Cannot spawn program: {}", x)),
                    };
                    let result = if self.spectate {
                        self.spectate(wsout, wsin, prog.stdin.take().expect("Cannot fail"))
                            .await
                    } else {
                        self.play(
                            wsout,
                            wsin,
                            prog.stdout.take().expect("Cannot fail"),
                            prog.stdin.take().expect("Cannot fail"),
                        )
                        .await
                    };
                    match result {
                        Ok(()) => match prog.wait().await {
                            Ok(x) if x.success() => Ok(()),
                            Ok(x) => Err(format!("Program exited with non-zero code: {}", x)),
                            Err(x) => Err(format!("Program exited abruptly: {}", x)),
                        },
                        Err(x) => {
                            drop(prog.kill().await);
                            Err(x)
                        }
                    }
                } else {
                    if self.spectate {
                        self.spectate(wsout, wsin, stdout()).await
                    } else {
                        let result = self.play(wsout, wsin, stdin(), stdout()).await;
                        println!("> Press ENTER to exit");
                        result
                    }
                }
            }
            #[cfg(unix)]
            CommunicationChannel::Pipe => {
                let pid = Pid::this();
                let dir = tempdir().map_err(|x| format!("Cannot create tempdir: {}", x))?;
                let inpipe_name = dir.path().join(format!("coco.{}.out", pid));
                let inp = inpipe_name.to_string_lossy();
                let outpipe_name = dir.path().join(format!("coco.{}.in", pid));
                let outp = outpipe_name.to_string_lossy();
                if !self.spectate {
                    mkfifo(&inpipe_name, Mode::S_IRWXU)
                        .map_err(|x| format!("Cannot create pipe: {}", x))?;
                }
                mkfifo(&outpipe_name, Mode::S_IRWXU)
                    .map_err(|x| format!("Cannot create pipe: {}", x))?;
                if self.program.len() > 0 {
                    let mut prog = proc::Command::new(&self.program[0]);
                    if self.program.len() > 1 {
                        prog.args(&self.program[1..]);
                    }
                    prog.env("COCO_PIPEIN", &outpipe_name);
                    if !self.spectate {
                        prog.env("COCO_PIPEOUT", &inpipe_name);
                    }
                    let mut prog = match prog.spawn() {
                        Ok(x) => x,
                        Err(x) => return Err(format!("Cannot spawn program: {}", x)),
                    };
                    if self.spectate {
                        println!("> Waiting until the program opens the pipe");
                    } else {
                        println!("> Waiting until the program open the pipes");
                    }
                    let result = if self.spectate {
                        let outpipe = OpenOptions::new()
                            .write(true)
                            .open(&outpipe_name)
                            .await
                            .map_err(|x| format!("Cannot open pipe: {}", x))?;
                        self.spectate(wsout, wsin, outpipe).await
                    } else {
                        let mut inpipe = OpenOptions::new();
                        let mut outpipe = OpenOptions::new();
                        let (inpipe, outpipe) = join!(
                            inpipe.read(true).open(&inpipe_name),
                            outpipe.write(true).open(&outpipe_name)
                        );
                        let (inpipe, outpipe) = match (inpipe, outpipe) {
                            (Ok(x), Ok(y)) => (x, y),
                            (Err(x), _) => return Err(format!("Cannot open pipe: {}", x)),
                            (_, Err(y)) => return Err(format!("Cannot open pipe: {}", y)),
                        };
                        self.play(wsout, wsin, inpipe, outpipe).await
                    };
                    match result {
                        Ok(()) => match prog.wait().await {
                            Ok(x) if x.success() => Ok(()),
                            Ok(x) => Err(format!("Program exited with non-zero code: {}", x)),
                            Err(x) => Err(format!("Program exited abruptly: {}", x)),
                        },
                        Err(x) => {
                            drop(prog.kill());
                            Err(x)
                        }
                    }
                } else {
                    if self.spectate {
                        println!("> Input (stdin-like) pipe: {}", outp);
                        println!("> Waiting until some program opens the pipe");
                        let outpipe = OpenOptions::new()
                            .write(true)
                            .open(&outpipe_name)
                            .await
                            .map_err(|x| format!("Cannot open pipe: {}", x))?;
                        self.spectate(wsout, wsin, outpipe).await
                    } else {
                        println!("> Input (stdin-like) pipe: {}", outp);
                        println!("> Output (stdout-like) pipe: {}", inp);
                        println!("> Waiting until some program open the pipes");
                        let mut inpipe = OpenOptions::new();
                        let mut outpipe = OpenOptions::new();
                        let (inpipe, outpipe) = join!(
                            inpipe.read(true).open(&inpipe_name),
                            outpipe.write(true).open(&outpipe_name)
                        );
                        let (inpipe, outpipe) = match (inpipe, outpipe) {
                            (Ok(x), Ok(y)) => (x, y),
                            (Err(x), _) => return Err(format!("Cannot open pipe: {}", x)),
                            (_, Err(y)) => return Err(format!("Cannot open pipe: {}", y)),
                        };
                        self.play(wsout, wsin, inpipe, outpipe).await
                    }
                }
            }
        }
    }

    async fn print_update(info: &MatchInfo, last: &mut HashSet<String>) {
        if info.connected != *last {
            *last = info.connected.clone();
            println!(
                "> Game has {} spectator{} and {}/{} ({} bot{}) connected player{}: {:?}",
                info.spectators,
                if info.spectators == 1 { "" } else { "s" },
                info.connected.len() + info.bots,
                info.players,
                info.bots,
                if info.bots == 1 { "" } else { "s" },
                if info.connected.len() == 1 { "" } else { "s" },
                info.connected.iter().collect::<Vec<_>>()
            );
        }
    }

    async fn play<
        T: Sink<Message> + Unpin,
        U: Stream<Item = Result<Message, TsError>> + Unpin,
        X: AsyncRead + Unpin,
        Y: AsyncWrite + Unpin,
    >(
        self,
        wsout: &mut T,
        wsin: &mut U,
        pipein: X,
        pipeout: Y,
    ) -> Result<(), String>
    where
        <T as Sink<Message>>::Error: Display,
    {
        let request = Request::LobbyJoinMatch {
            id: self.id.clone(),
            name: self.name.clone().unwrap_or_else(|| whoami::username()),
            password: self.password.clone(),
        };
        let info = match oneshot_request(request, wsout, wsin).await {
            Ok(Reply::LobbyJoinedMatch { info: Ok(x) }) => x,
            Ok(Reply::LobbyJoinedMatch { info: Err(x) }) => {
                return Err(format!("Cannot join game: {}", x))
            }
            Ok(_) => return Err(format!("Server sent wrong reply")),
            Err(x) => return Err(x),
        };
        let mut last_connected = HashSet::new();
        println!("> Joined \"{}\" ({})", info.name, info.game);
        println!("> Waiting for game to start");
        Self::print_update(&info, &mut last_connected).await;
        loop {
            let msg = match wsin.next().await {
                Some(x) => x,
                None => break Err(format!("Connection lost")),
            };
            let msg = match msg {
                Ok(Message::Text(x)) => x,
                Ok(_) => continue,
                Err(x) => break Err(format!("Connection lost: {}", x)),
            };
            match Reply::parse(&msg) {
                Ok(Reply::MatchStarted {}) => {
                    println!("> Game started");
                    break self.ingame(wsout, wsin, pipein, pipeout).await;
                }
                Ok(Reply::LobbyUpdate { info }) => {
                    Self::print_update(&info, &mut last_connected).await
                }
                Ok(Reply::LobbyDelete { .. }) => break Err(format!("Game expired")),
                Ok(_) => break Err(format!("Received wrong message from server: {:?}", msg)),
                Err(x) => break Err(format!("Cannot parse server reply: {}", x)),
            }
        }
    }

    async fn ingame<
        T: Sink<Message> + Unpin,
        U: Stream<Item = Result<Message, TsError>> + Unpin,
        X: AsyncRead + Unpin,
        Y: AsyncWrite + Unpin,
    >(
        self,
        wsout: &mut T,
        wsin: &mut U,
        mut pipein: X,
        mut pipeout: Y,
    ) -> Result<(), String>
    where
        <T as Sink<Message>>::Error: Display,
    {
        let mut buffer = [0; BUFFER_SIZE];
        loop {
            select! {
                msg = wsin.next() => {
                    let msg = match msg {
                        Some(Ok(Message::Text(x))) => x,
                        Some(Ok(Message::Binary(x))) => {
                            if let Err(x) = pipeout.write_all(&x).await {
                                break Err(format!("Cannot write to stream: {}", x));
                            }
                            if let Err(x) = pipeout.flush().await {
                                warn!("Cannot flush stream: {}", x);
                            }
                            continue;
                        }
                        Some(Ok(_)) => continue,
                        Some(Err(x)) => break Err(format!("Connection lost: {}", x)),
                        None => break Err(format!("Connection lost")),
                    };
                    match Reply::parse(&msg) {
                        Ok(Reply::MatchEnded {}) => {
                            println!("> Game ended");
                            break Ok(());
                        }
                        Ok(Reply::LobbyUpdate { .. }) => {}
                        Ok(_) => break Err(format!("Received wrong message from server: {:?}", msg)),
                        Err(x) => break Err(format!("Cannot parse server reply: {}", x)),
                    }
                }
                res = pipein.read(&mut buffer) => {
                    let size = match res {
                        Ok(0) => {
                            warn!("Read 0 bytes from stream");
                            continue;
                        }
                        Ok(x) => x,
                        Err(x) => break Err(format!("Cannot read from stream: {}", x)),
                    };
                    if let Err(x) = wsout.send(Message::Binary(buffer[..size].into())).await {
                        break Err(format!("Cannot send game data to server: {}", x));
                    }
                }
            }
        }
    }

    async fn spectate<
        T: Sink<Message> + Unpin,
        U: Stream<Item = Result<Message, TsError>> + Unpin,
        Y: AsyncWrite + Unpin,
    >(
        self,
        wsout: &mut T,
        wsin: &mut U,
        mut pipeout: Y,
    ) -> Result<(), String>
    where
        <T as Sink<Message>>::Error: Display,
    {
        let reply = oneshot_request(Request::SpectateJoin { id: self.id }, wsout, wsin);
        let info = match reply.await {
            Ok(Reply::SpectateJoined { info: Ok(x) }) => x,
            Ok(Reply::SpectateJoined { info: Err(x) }) => {
                return Err(format!("Spectate join failed: {}", x))
            }
            Ok(_) => return Err(format!("Server sent wrong reply")),
            Err(x) => return Err(x),
        };
        let mut last_connected = HashSet::new();
        println!("> Joined spectators for \"{}\" ({})", info.name, info.game);
        println!("> Waiting for game to start");
        Self::print_update(&info, &mut last_connected).await;
        loop {
            let msg = match wsin.next().await {
                Some(x) => x,
                None => break Err(format!("Connection lost")),
            };
            let msg = match msg {
                Ok(Message::Text(x)) => x,
                Ok(Message::Binary(x)) => {
                    if let Err(x) = pipeout.write_all(&x).await {
                        break Err(format!("Cannot write to spectator stream: {}", x));
                    }
                    if let Err(x) = pipeout.flush().await {
                        warn!("Cannot flush to spectator stream: {}", x);
                    }
                    continue;
                }
                Ok(_) => continue,
                Err(x) => break Err(format!("Connection lost: {}", x)),
            };
            match Reply::parse(&msg) {
                Ok(Reply::SpectateStarted {}) => println!("> Game started"),
                Ok(Reply::SpectateSynced {}) => println!("> Reached current game status"),
                Ok(Reply::SpectateEnded {}) | Ok(Reply::SpectateLeaved {}) => {
                    println!("> Game ended");
                    break Ok(());
                }
                Ok(Reply::LobbyUpdate { info }) => {
                    Self::print_update(&info, &mut last_connected).await
                }
                Ok(Reply::LobbyDelete { .. }) => break Err(format!("Game expired")),
                Ok(_) => break Err(format!("Received wrong message from server: {:?}", msg)),
                Err(x) => break Err(format!("Cannot parse server reply: {}", x)),
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
    let result = match ws.get_mut() {
        &mut MaybeTlsStream::Plain(ref mut x) => x.set_nodelay(true),
        &mut MaybeTlsStream::Rustls(ref mut x) => x.get_mut().0.set_nodelay(true),
        &mut _ => unreachable!("Using stream other than Plain or Rustls"),
    };
    if let Err(x) = result {
        warn!("Cannot set TCP_NODELAY: {}", x);
    }
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

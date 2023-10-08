use crate::game;
use async_trait::async_trait;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};
use tracing::error;

use std::thread;
use std::time::Duration;

use super::board::ChessBoard;
use super::chess_move::MoveType;
use super::color::Color;

#[derive(Debug)] // The Bot struct is defined and derived with the Debug trait, allowing it to be printed for debugging purposes.
pub(crate) struct Bot {}

#[async_trait] // The Bot struct implements the game::Bot trait using the async_trait macro. This trait defines methods related to game bots.
impl game::Bot for Bot {
    // This asynchronous function is the entry point for the bot's execution. It takes a DuplexStream as an argument, which provides bidirectional communication between the bot and the game server.
    async fn start(&mut self, stream: DuplexStream) {
        let mut board = ChessBoard::new();
        let mut current_color = Color::White;

        // The DuplexStream is split into separate reader and writer halves (input and output, respectively). This allows the bot to read data from the server and send responses back.
        let (input, mut output) = split(stream);
        let mut input = BufReader::new(input);

        // The bot reads some initial data from the server, such as its name, the opponent's name, and its player number (me).
        lnin!(input); // Read my name
        lnin!(input); // Read opponent name
        let me: usize = lnin!(input).parse().expect("Cannot parse player number");
        let mut turn = 0;

        // The bot enters a game loop that continues until the game is finished (determined by the finished() method of the Board).
        while !board.check_king_mate(current_color) {
            if turn == 0 {
                let mut current_color = Color::White;
            } else {
                let mut current_color = Color::Black;
            }

            let mut trimmed = String::with_capacity(10);

            let mut opt = MoveType::parse(&trimmed);

            // Inside the game loop, when it is the bot's turn (indicated by turn == me), it receives a roll value from the server. It then generates a move based on the board.valid_moves(me, roll) method. The bot selects a random valid move using choose() from the valid_moves slice, and then it sends its move to the server using lnout!(output, format!("{}", x)).
            if turn == me {
                let mut user_input = String::new();

                while !board.check_move(opt, current_color) {
                    trimmed = MoveType::randomMove();
                    opt = MoveType::parse(&trimmed);
                }
                board = board.apply_move_type(opt.expect("Invalid MoveType received"));
                // lnout!(output, format!("{}", trimmed));
            } else {
                // When it is the opponent's turn (turn != me), the bot reads the opponent's move from the server. If the opponent sends "RETIRE," the game breaks out of the loop, otherwise, it parses the move and updates the board accordingly.
                trimmed = lnin!(input)
                    .trim()
                    .split(" ")
                    .map(|x| x.to_string())
                    .collect();

                if board.check_move(opt, current_color) {
                    board = board.apply_move_type(opt.expect("Invalid MoveType received"));
                    turn = 1 - turn;
                } else {
                    // lnout!(output, format!("INVALID_MOVE <Server sent invalid move {}>", trimmed));
                }
            }
            // After each move, the turn variable is flipped (turn = 1 - turn) to switch between the bot's turn and the opponent's turn.
            turn = 1 - turn;
        }
        thread::sleep(Duration::from_secs(3));
    }
}

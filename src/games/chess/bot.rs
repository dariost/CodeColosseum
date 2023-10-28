use crate::game;
extern crate regex;

use regex::Regex;
use async_trait::async_trait;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};
use tracing::error;
use tracing::warn;

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
    	//println!(">>> BOT: Io sono il giocatore {}", me);
    	
        let mut current_color = Color::White;

        // The bot enters a game loop that continues until the game is finished (determined by the finished() method of the Board).
        while !board.check_king_mate(current_color) {
        	//println!(">>> BOT: Turno {}", turn);
            if turn == 0 {
                current_color = Color::White;
            } else {
                current_color = Color::Black;
            }
	        	
        	/*match current_color{
        		Color::Black => println!(">>> BOT: Nero"),
        		Color::White => println!(">>> BOT: Bianco"),
        	}*/

            let mut trimmed = String::with_capacity(10);

            let mut opt = MoveType::parse(&trimmed);

            if turn == me {
	        	//println!(">>> BOT: Calcolo la mossa");

                while !board.check_move(opt, current_color) {
                    trimmed = MoveType::randomMove();
                    opt = MoveType::parse(&trimmed);
		        	//println!(">>> BOT: Mossa casuale {}", trimmed);
                }
                board = board.apply_move_type(opt.expect("Invalid MoveType received"));
                lnout!(output, format!("{}", trimmed));
                //println!(">>> BOT: Mossa inviata {}", trimmed);
            } else {
                // When it is the opponent's turn (turn != me), the bot reads the opponent's move from the server. If the opponent sends "RETIRE," the game breaks out of the loop, otherwise, it parses the move and updates the board accordingly.
                let mut token = lnin!(input);
                    
                let re = Regex::new(r"<(.*?)>").expect("Failed to compile regex pattern");
				let captures: Vec<&str> = re.captures_iter(&token)
					.filter_map(|cap| cap.get(1).map(|m| m.as_str()))
					.collect();
				trimmed = captures.join(" ");
                opt = MoveType::parse(&trimmed);
                
                //println!(">>> BOT: Mossa ricevuta {}", trimmed);
                    
                if (trimmed.len() <= 0) | (trimmed == "RETIRE"){
                	break;
                }

                if board.check_move(opt, current_color) {
                	//println!(">>> BOT: Mossa valida, cambio turno");
                    board = board.apply_move_type(opt.expect("Invalid MoveType received"));
                } else {
                	//println!(">>> BOT: Mossa non valida");
                    lnout!(output, format!("INVALID_MOVE <Server sent invalid move {}>", trimmed));
                    turn = 1 - turn;
                    break;
                }
            }
            // After each move, the turn variable is flipped (turn = 1 - turn) to switch between the bot's turn and the opponent's turn.
            turn = 1 - turn;
        }
    }
}

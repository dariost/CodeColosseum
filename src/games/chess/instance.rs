// Import necessary dependencies and modules
use super::super::util::Player;
use crate::game;
use async_trait::async_trait;
use rand::rngs::StdRng;
use rand::Rng;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, DuplexStream, WriteHalf};
use tokio::time::{sleep_until, timeout, Duration, Instant};
use tracing::warn;
use tracing::error;

use super::board::ChessBoard;
use super::chess_move::MoveType;
use super::color::Color;

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;

// Define a struct named 'Instance' to hold game-related parameters
#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) timeout: Duration,
    pub(crate) pace: Duration,
    pub(crate) rng: StdRng,
}

// Define a macro 'retired!' which sends a retirement message to players and spectators
macro_rules! retired {
    ($other:expr, $spectators:expr) => {{
        lnout!($other, "RETIRE");
        lnout!($spectators, "RETIRE");
        break;
    }};
}

// Function to refresh the player's turn color and print the corresponding message
pub fn refreshColor(turn: usize) -> Color {
    let mut current_color = Color::Black;
    if turn == 0 {
        current_color = Color::White;
        //println!("White's turn");
    } else {
        //println!("Black's turn");
    }
    current_color
}

// Implement the game::Instance trait for the defined Instance struct
#[async_trait]
impl game::Instance for Instance {
    // Define the 'start' method required by the trait
    async fn start(
        &mut self,
        players: HashMap<String, DuplexStream>,
        mut spectators: WriteHalf<DuplexStream>,
    ) {
        // Initialize the chess board and players
        let mut board = ChessBoard::new();
        let mut p = Player::from(players, &mut self.rng);
        assert_eq!(p.len(), 2);

        // Send player names to all participants
        for i in 0..2 {
            lnout!(p[0].output, &p[i].name);
            lnout!(p[1].output, &p[i].name);
            lnout!(spectators, &p[i].name);
        }
        // Send player index to players
        lnout!(p[0].output, "0");
        lnout!(p[1].output, "1");

        let mut turn = 0;
        let mut turn_prec = 1;
        let mut current_color = Color::White;
        let mut retired = 0;
        let mut draw = 0;
        let mut errorCount = 0;
        let mut hide_output = 0;

        // Main game loop
        while !board.check_king_mate(current_color) && retired == 0 && draw != 2 {
        	//println!(">>> PLAYER: Turno {}, turno precedente {}", turn, turn_prec);
            if turn != turn_prec {
                board.display();
                turn_prec = turn;
                errorCount = 0;
            } else {
                errorCount = errorCount + 1;
            };

            if errorCount == 10000 {
                retired = 1;
                retired!(p[1 - turn].output, spectators)
            }

            let start = Instant::now();

			//------------------------------------------------------------------------------------
            // Read move
            let mut buffer = String::new();
            let mut token = "";
        	//println!(">>> PLAYER: Attendo la mossa del giocatore {}", turn);
            match timeout(self.timeout, p[turn].input.read_line(&mut buffer)).await {
                // Timed out or closed connection
                Err(_) | Ok(Err(_)) => retired!(p[1 - turn].output, spectators),
                // Parse response
                Ok(Ok(_)) => if buffer.trim().len() > 0{
						        // Valid token value
						        token = buffer.trim();
					       	} else {
						        // Other garbage
						        retired = 1;
						        hide_output = 1;
						        retired!(p[1 - turn].output, spectators);
						        continue;
						    }
            };
            let mut trimmed = String::new();
            trimmed = token.to_string();
            
            //println!(">>> PLAYER: Mossa ricevuta {}", trimmed);
            
            if trimmed.len() <= 0 {
			    retired = 1;
			    hide_output = 1;
			    retired!(p[1 - turn].output, spectators);
			    continue;
            };

            // Handle the draw condition
            if draw == 1 {
                if trimmed == "DRAW" {
                    //lnout!(p[turn].output, "DRAW <ACCEPTED>");
                    lnout!(p[1 - turn].output, "DRAW <ACCEPTED>");
                    lnout!(spectators, "DRAW <ACCEPTED>");
                    draw = draw + 1;
                } else {
                    draw = 0;
                    //lnout!(p[turn].output, "DRAW <REFUSED>");
                    lnout!(p[1 - turn].output, "DRAW <REFUSED>");
                    lnout!(spectators, "DRAW <REFUSED>");
                    turn = 1 - turn;
                    current_color = refreshColor(turn);
                };
                continue;
            };

            // Split the input string into words
            let words: Vec<&str> = trimmed.split_whitespace().collect();

            // Create a new string with < > around each word and join them
            let formatted_str = words
                .iter()
                .map(|&word| format!("<{}>", word))
                .collect::<Vec<String>>()
                .join(" ");

            // Process the player's move
            let opt = MoveType::parse(&trimmed);
            if !board.check_move(opt, current_color) {
                if trimmed == "RETIRE" {
                    retired = 1;
                    retired!(p[1 - turn].output, spectators)
                } else {
                    if trimmed == "DRAW" {
                        draw = 1;
                        lnout!(p[turn].output, "DRAW <PROPOSED>");
                        lnout!(p[1 - turn].output, "DRAW <PROPOSED>");
                        lnout!(spectators, "DRAW <PROPOSED>");
                        turn = 1 - turn;
                    } else {
			            //println!(">>> PLAYER: Mossa ricevuta non valida");
                        /*lnout!(
                            p[turn].output,
                            "INVALID_MOVE <Impossible to recognize move>"
                        );*/
                    }
                };
                continue;
            } else {
	            //println!(">>> PLAYER: Mossa valida");
            	let mut out_str = "OK ".to_owned() + &formatted_str;
                lnout!(p[turn].output, "OK ".to_owned() + &formatted_str);
                lnout!(p[1 - turn].output, "OK ".to_owned() + &formatted_str);
                lnout!(spectators, "OK ".to_owned() + &formatted_str);

                match opt {
                    Some(move_type) => {
                        board = board.apply_move_type(move_type);
                    }
                    None => {
                        lnout!(p[turn].output, "INVALID_MOVE <Impossible to apply move>");
                        continue;
                    }
                }
                turn = 1 - turn;
                current_color = refreshColor(turn);
                sleep_until(start + self.pace).await;
                continue;
            }
        }

        // Game ending messages
        if draw != 2 && hide_output == 0 {
            lnout!(p[1 - turn].output, "CHECKMATE! You win!");
            lnout!(p[turn].output, "CHECKMATE! You loose!");
            if turn == 0 {
                lnout!(spectators, "CHECKMATE! Black wins!");
            } else {
                lnout!(spectators, "CHECKMATE! White wins!");
            };
        }
    }

    async fn args(&self) -> HashMap<String, String> {
        HashMap::from([("pace".to_owned(), format!("{:?}", self.pace))])
    }
}

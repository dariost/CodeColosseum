// Importing necessary external crates
use super::board::Piece;
use rand::Rng; // Importing the random number generator trait

// Defining a Point struct to represent a position on the board
#[derive(Copy, Clone, PartialEq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    // Implementing a method to parse a string input into a Point
    fn parse(input: &str) -> Option<Point> {
        // Checking if the input has a length of 2 characters
        if input.len() != 2 {
            None // Return None if input length is not 2
        } else {
            let (x, y) = (input.chars().nth(0).unwrap(), input.chars().nth(1).unwrap());

            // Checking if the characters represent a valid chess position
            if x < 'a' || x > 'h' || y < '1' || y > '8' {
                None // Return None if x or y is out of valid range
            } else {
                Some(Point {
                    // Parsing the characters into x and y coordinates
                    x: x.to_digit(20).unwrap() as usize - 10,
                    y: y.to_digit(10).unwrap() as usize - 1,
                })
            }
        }
    }
}

// Defining a Move struct to represent a chess move
#[derive(Copy, Clone, PartialEq)]
pub struct Move {
    pub from: Point,
    pub to: Point,
}

impl Move {
    // Implementing a method to parse two string inputs into a Move
    fn parse(from: &str, to: &str) -> Option<Move> {
        match (Point::parse(from), Point::parse(to)) {
            (_, None) | (None, _) => None, // Return None if either from or to points couldn't be parsed
            (Some(from), Some(to)) => Some(Move { from, to }), // Create a Move if both points are successfully parsed
        }
    }
}

// Defining an enum MoveType to represent different types of chess moves
#[derive(Copy, Clone)]
pub enum MoveType {
    Basic(Move),
    EnPassant(Move),
    Promotion(Move, Piece),
    Castling(Move, Move),
}

impl MoveType {
    // Implementing a method to generate a random chess move in string format
    pub fn randomMove() -> String {
        let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']; // Array of file characters
        let ranks = ['1', '2', '3', '4', '5', '6', '7', '8']; // Array of rank characters

        let mut rng = rand::thread_rng(); // Creating a random number generator instance
        let random_file_from = rng.gen_range(0..8); // Generating a random file index
        let random_rank_from = rng.gen_range(0..8); // Generating a random rank index
        let random_file_to   = rng.gen_range(0..8); // Generating a random file index
        let random_rank_to   = rng.gen_range(0..8); // Generating a random rank index

        format!("{}{} {}{}", files[random_file_from], ranks[random_rank_from], files[random_file_to], ranks[random_rank_to]) // Formatting and returning the random move string
    }

    // Implementing a method to parse a string input into a MoveType enum
    pub fn parse(input: &str) -> Option<MoveType> {
        let mut words = input.split(" "); // Splitting the input string into words
        match input.split(" ").count() {
            // Matching the count of words in the input
            2 => match Move::parse(words.next().unwrap(), words.next().unwrap()) {
                None => None,
                Some(mv) => Some(MoveType::Basic(mv)),
            },
            3 => {
                if words.next().unwrap() != "enpassant" {
                    None
                } else {
                    match Move::parse(words.next().unwrap(), words.next().unwrap()) {
                        None => None,
                        Some(mv) => Some(MoveType::EnPassant(mv)),
                    }
                }
            },
            4 => {
                if words.next().unwrap() != "promote" {
                    None
                } else {
                    match (
                        Move::parse(words.next().unwrap(), words.next().unwrap()),
                        Piece::parse(words.next().unwrap()),
                    ) {
                        (_, None) | (None, _) => None,
                        (Some(mv), Some(piece)) => Some(MoveType::Promotion(mv, piece)),
                    }
                }
            },
            5 => {
                if words.next().unwrap() != "castle" {
                    None
                } else {
                    match (
                        Move::parse(words.next().unwrap(), words.next().unwrap()),
                        Move::parse(words.next().unwrap(), words.next().unwrap()),
                    ) {
                        (_, None) | (None, _) => None,
                        (Some(mvk), Some(mvr)) => Some(MoveType::Castling(mvk, mvr)),
                    }
                }
            },
            _ => None,
        }
    }
}

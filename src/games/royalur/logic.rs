#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Player(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Token(Player, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Position {
    Start(Player),
    Board(usize, usize),
    End(Player),
}

#[derive(Debug)]
pub(crate) struct Board {
    board: [[Option<Token>; 8]; 3],
    position: [[Position; 7]; 2],
}

impl Board {
    pub(crate) fn new() -> Board {
        Board {
            board: [[None; 8]; 3],
            position: [
                [Position::Start(Player(0)); 7],
                [Position::Start(Player(1)); 7],
            ],
        }
    }

    fn advance_once(pos: Option<Position>, player: usize) -> Option<Position> {
        let pos = match pos {
            Some(x) => x,
            None => return None,
        };
        match (pos, player) {
            (Position::End(_), _) => None,
            (Position::Board(0, 0), _) | (Position::Board(2, 0), _) => Some(Position::Board(1, 0)),
            (Position::Board(0, 6), p) | (Position::Board(2, 6), p) => {
                Some(Position::End(Player(p)))
            }
            (Position::Board(1, 7), p) => Some(Position::Board(p * 2, 7)),
            (Position::Board(1, c), _) => Some(Position::Board(1, c + 1)),
            (Position::Board(r, c), _) => Some(Position::Board(r, c - 1)),
            (Position::Start(Player(p)), _) => Some(Position::Board(p * 2, 3)),
        }
    }

    fn advance(pos: Position, dist: usize, player: usize) -> Option<Position> {
        let mut pos = Some(pos);
        for _ in 0..dist {
            pos = Self::advance_once(pos, player);
        }
        pos
    }

    fn simulate_move(&self, player: usize, token: usize, dist: usize) -> Option<Position> {
        let pos = self.position[player][token];
        match Self::advance(pos, dist, player) {
            Some(pos) => match pos {
                Position::Start(_) => None,
                Position::Board(row, col) => match self.board[row][col] {
                    Some(Token(Player(p), _)) => {
                        if p == player || (row == 1 && col == 3) {
                            None
                        } else {
                            Some(Position::Board(row, col))
                        }
                    }
                    None => Some(Position::Board(row, col)),
                },
                Position::End(p) => Some(Position::End(p)),
            },
            None => None,
        }
    }

    pub(crate) fn valid_moves(&self, player: usize, dist: usize) -> Vec<usize> {
        assert!(player < 2);
        assert!(dist <= 4);
        (0..7)
            .filter(|&x| self.simulate_move(player, x, dist).is_some())
            .collect()
    }

    pub(crate) fn make_move(
        &mut self,
        player: usize,
        token: usize,
        dist: usize,
    ) -> Result<bool, String> {
        assert!(player < 2);
        assert!(dist <= 4);
        if token > 7 {
            return Err(format!("Invaid token number {} > 7", token));
        }
        if let Some(pos) = self.simulate_move(player, token, dist) {
            match pos {
                Position::End(p) => {
                    let (row, col) = match self.position[player][token] {
                        Position::Board(r, c) => (r, c),
                        _ => unreachable!("To end it must be on the board"),
                    };
                    self.board[row][col] = None;
                    self.position[player][token] = Position::End(p);
                    Ok(false)
                }
                Position::Board(row, col) => match self.position[player][token] {
                    Position::Board(r, c) => {
                        if let Some(Token(Player(p), t)) = self.board[row][col] {
                            self.position[p][t] = Position::Start(Player(p));
                        }
                        self.board[r][c] = None;
                        self.board[row][col] = Some(Token(Player(player), token));
                        self.position[player][token] = Position::Board(row, col);
                        Ok((row == 0 && col == 0)
                            || (row == 2 && col == 0)
                            || (row == 1 && col == 3)
                            || (row == 0 && col == 6)
                            || (row == 2 && col == 6))
                    }
                    Position::Start(p) => {
                        assert_eq!(self.board[row][col], None);
                        self.board[row][col] = Some(Token(p, token));
                        self.position[player][token] = Position::Board(row, col);
                        Ok(col == 0)
                    }
                    _ => unreachable!("Cannot move from end"),
                },
                Position::Start(_) => Err(format!("Cannot move to start")),
            }
        } else {
            Err(format!("Invalid move"))
        }
    }

    pub(crate) fn finished(&self) -> bool {
        self.position.iter().any(|x| {
            x.iter().all(|&y| {
                if let Position::End(_) = y {
                    true
                } else {
                    false
                }
            })
        })
    }
}

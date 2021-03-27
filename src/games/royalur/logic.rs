#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Token {
    P0(usize),
    P1(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Position {
    Start,
    Board(usize, usize),
    End,
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
            position: [[Position::Start; 7]; 2],
        }
    }

    pub(crate) fn valid_moves(&self, player: usize, dist: usize) -> Vec<usize> {
        assert!(player < 2);
        assert!(dist <= 4);
        todo!()
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
        todo!()
    }

    pub(crate) fn finished(&self) -> bool {
        self.position
            .iter()
            .any(|x| x.iter().all(|&y| y == Position::End))
    }
}

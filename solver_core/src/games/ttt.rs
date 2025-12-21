use crate::{
    game::{GameState, Player},
    utils::opposite_player,
};

const WIN_LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

/// Represents the contents of a single Tic-Tac-Toe board cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// The cell is empty (no player has played here yet).
    Empty,
    /// The cell is occupied by Player1 (we'll treat this as 'X').
    X,
    /// The cell is occupied by Player2 (we'll treat this as 'O').
    O,
}

/// Represents a full Tic-Tac-Toe game state.
///
/// This struct stores:
/// - the current board position as an array of 9 cells,
/// - whose turn it is to move.
///
/// Indexing convention (recommended):
///  0 | 1 | 2
/// ---+---+---
///  3 | 4 | 5
/// ---+---+---
///  6 | 7 | 8
#[derive(Clone, Debug)]
pub struct TicTacToeState {
    /// The 3x3 board flattened into a fixed-size array of 9 cells.
    pub board: [Cell; 9],
    /// The player whose turn it is to move in this position.
    pub current_player: Player,
}

impl Default for TicTacToeState {
    fn default() -> Self {
        Self::new()
    }
}

impl TicTacToeState {
    /// Creates a new game state representing the standard initial position:
    /// - all cells are empty,
    /// - Player1 is to move.
    pub fn new() -> Self {
        Self {
            board: [Cell::Empty; 9],
            current_player: Player::Player1,
        }
    }

    /// Attempts to construct a TicTacToeState from a string representation.
    ///
    /// Suggested format (but you can choose your own as long as you're consistent):
    /// - A string of length 9.
    /// - Each character is one of: 'X', 'O', or '.' (for empty).
    /// - Example: "XOX...O.." means:
    ///   X | O | X
    ///   . | . | .
    ///   O | . | .
    ///
    /// The current player could be inferred (e.g., X if #X == #O, else O),
    /// or you can decide to keep it simple and pass the current player in
    /// as an argument in a later version.
    pub fn from_str(repr: &str, current_player: Player) -> Result<Self, String> {
        let cells: Vec<Cell> = repr
            .chars()
            .map(|n| match n {
                'X' => Ok(Cell::X),
                'O' => Ok(Cell::O),
                '.' => Ok(Cell::Empty),
                _ => Err(format!("Invalid character: {}", n)),
            })
            .collect::<Result<_, _>>()?;

        let board: [Cell; 9] = cells
            .try_into()
            .map_err(|v: Vec<_>| format!("Expected 9 cells, got {}", v.len()))?;

        Ok(Self {
            board,
            current_player,
        })
    }
}

/// Represents a single move in Tic-Tac-Toe.
///
/// For simplicity, a move is just "play in this cell index".
/// The index should be in the range 0..=8, using the same
/// indexing convention as `TicTacToeState`.
#[derive(Clone, Copy, Debug)]
pub struct TicTacToeMove {
    /// The index (0..=8) of the cell where the current player plays.
    pub index: u8,
}

impl GameState for TicTacToeState {
    /// The move type for Tic-Tac-Toe is just a cell index (0..=8).
    type Move = TicTacToeMove;

    /// Returns the player whose turn it is in this game state.
    ///
    /// For Tic-Tac-Toe, this is stored directly in the `current_player` field.
    fn current_player(&self) -> Player {
        self.current_player
    }

    /// Returns a list of all legal moves from this position.
    ///
    /// A move is legal if:
    /// - its index is in the range 0..=8, and
    /// - the corresponding cell on the board is `Cell::Empty`.
    fn legal_moves(&self) -> Vec<Self::Move> {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(i, cell)| {
                if *cell == Cell::Empty {
                    Some(Self::Move { index: i as u8 })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Applies the given move and returns the resulting game state.
    ///
    /// This method MUST:
    /// - NOT mutate `self`
    /// - Create a new board (copying `self.board`)
    /// - Update the chosen cell based on the current player
    /// - Switch the current player to the opponent
    ///
    /// Assumptions (for now):
    /// - The move is legal.
    /// - The index is within 0..=8.
    /// - The target cell is empty.
    ///
    /// Later, we might add optional validation or debug assertions.
    fn apply_move(&self, mv: &Self::Move) -> Self {
        let mut new_board = self.board;
        let mark = match self.current_player {
            Player::Player1 => Cell::X,
            Player::Player2 => Cell::O,
        };
        new_board[mv.index as usize] = mark;
        let new_player = opposite_player(self.current_player);
        Self {
            board: new_board,
            current_player: new_player,
        }
    }

    /// Returns true if this position is terminal (win or draw),
    /// and false otherwise.
    fn is_terminal(&self) -> bool {
        if WIN_LINES.iter().any(|&[a, b, c]| {
            self.board[a] == self.board[b]
                && self.board[b] == self.board[c]
                && self.board[a] != Cell::Empty
        }) {
            return true;
        }
        // Check for draw: no win and all cells filled
        self.board.iter().all(|cell| *cell != Cell::Empty)
    }

    /// Returns the utility value of this state if it is terminal.
    ///
    /// Convention (from Player1's perspective):
    /// - +1 for a Player1 (X) win,
    /// - -1 for a Player2 (O) win,
    /// -  0 for a draw,
    /// - None if the state is not terminal.
    fn terminal_value(&self) -> Option<i32> {
        for &[a, b, c] in WIN_LINES.iter() {
            if self.board[a] == self.board[b]
                && self.board[b] == self.board[c]
                && self.board[a] != Cell::Empty
            {
                return Some(match self.board[a] {
                    Cell::X => 1,
                    Cell::O => -1,
                    Cell::Empty => unreachable!(),
                });
            }
        }
        if self.board.iter().all(|cell| *cell != Cell::Empty) {
            return Some(0);
        }
        None
    }

    fn move_ordering_key(&self, mv: &TicTacToeMove) -> i32 {
        match mv.index {
            4 => 3,             // center
            0 | 2 | 6 | 8 => 2, // corners
            _ => 1,             // edges
        }
    }
}

fn cell_to_char(c: Cell) -> char {
    match c {
        Cell::Empty => '.',
        Cell::X => 'X',
        Cell::O => 'O',
    }
}

/// Pretty-prints a Tic-Tac-Toe state to stdout.
///
/// Display convention (indices in comments):
///   0 | 1 | 2
///  ---+---+---
///   3 | 4 | 5
///  ---+---+---
///   6 | 7 | 8
///
/// Suggested display:
///   X | O | .
///  ---+---+---
///   . | X | .
///  ---+---+---
///   O | . | .
pub fn print_ttt_board(state: &TicTacToeState) {
    for row in 0..3 {
        let base = row * 3;
        let a = cell_to_char(state.board[base]);
        let b = cell_to_char(state.board[base + 1]);
        let c = cell_to_char(state.board[base + 2]);
        println!("{a} | {b} | {c}");

        if row < 2 {
            println!("---+---+---");
        }
    }
}

/// Parses a user input string into a TicTacToeMove.
///
/// Expected format:
/// - A single digit "0".."8" for direct index
///
/// This function will:
/// - Trim whitespace
/// - Return Err(...) on malformed input
/// - Return Err(...) if the chosen cell is not empty in `state`
pub fn parse_ttt_move(input: &str, state: &TicTacToeState) -> Result<TicTacToeMove, String> {
    let clean = input.trim();
    // Try to parse as usize
    let idx: usize = clean
        .parse()
        .map_err(|_| "Could not parse input as a number in 0..=8".to_string())?;
    if idx > 8 {
        return Err("Index must be between 0 and 8".to_string());
    }
    if state.board[idx] != Cell::Empty {
        return Err("Cell is not empty.".to_string());
    }
    Ok(TicTacToeMove { index: idx as u8 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::minimax::{minimax_value, minimax_value_ab_root};

    #[test]
    fn ttt_from_str_parses_correctly() {
        let s = TicTacToeState::from_str("X.O...O..", Player::Player1).unwrap();
        assert_eq!(s.board[0], Cell::X);
        assert_eq!(s.board[1], Cell::Empty);
        assert_eq!(s.board[2], Cell::O);
        assert_eq!(s.board[7], Cell::Empty);
        assert_eq!(s.current_player, Player::Player1);
    }

    #[test]
    fn ttt_from_str_rejects_invalid_char() {
        let res = TicTacToeState::from_str("X.Z...O..", Player::Player1);
        assert!(res.is_err());
    }

    #[test]
    fn ttt_from_str_rejects_wrong_length() {
        let too_short = TicTacToeState::from_str("XOX.....", Player::Player1);
        assert!(too_short.is_err());

        let too_long = TicTacToeState::from_str("XOX...O..X", Player::Player1);
        assert!(too_long.is_err());
    }

    #[test]
    fn ttt_terminal_value_x_win_row() {
        // X X X
        // . . .
        // . . .
        let s = TicTacToeState::from_str("XXX......", Player::Player2).unwrap();
        assert!(s.is_terminal());
        assert_eq!(s.terminal_value(), Some(1));
    }

    #[test]
    fn ttt_terminal_value_o_win_diag() {
        // X O .
        // X O .
        // . O .
        let s = TicTacToeState::from_str("XO.XO..O.", Player::Player1).unwrap();
        assert!(s.is_terminal());
        assert_eq!(s.terminal_value(), Some(-1));
    }

    #[test]
    fn ttt_terminal_value_draw() {
        // X O X
        // X O O
        // O X X
        let s = TicTacToeState::from_str("XOXXOOOXX", Player::Player1).unwrap();
        assert!(s.is_terminal());
        assert_eq!(s.terminal_value(), Some(0));
    }

    #[test]
    fn ttt_non_terminal_has_no_terminal_value() {
        let s = TicTacToeState::from_str("X.O...O..", Player::Player2).unwrap();
        assert!(!s.is_terminal());
        assert_eq!(s.terminal_value(), None);
    }

    #[test]
    fn ttt_minimax_value_start_position_is_draw() {
        let s = TicTacToeState::new();
        // Perfect play from both sides should be a draw
        assert_eq!(minimax_value(&s), 0);
        assert_eq!(minimax_value_ab_root(&s), 0);
    }

    #[test]
    fn ttt_minimax_and_ab_agree_on_random_position() {
        // X O X
        // . X .
        // O . .
        let s = TicTacToeState::from_str("XOX.XO...", Player::Player2).unwrap();
        let v1 = minimax_value(&s);
        let v2 = minimax_value_ab_root(&s);
        assert_eq!(v1, v2);
    }

    #[test]
    fn ttt_move_ordering_prefers_center_over_corner() {
        let s = TicTacToeState::new();
        let center = TicTacToeMove { index: 4 };
        let corner = TicTacToeMove { index: 0 };
        assert!(s.move_ordering_key(&center) > s.move_ordering_key(&corner));
    }
}

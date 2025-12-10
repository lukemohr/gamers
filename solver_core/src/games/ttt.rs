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

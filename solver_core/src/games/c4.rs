use crate::game::{GameState, Player};
use crate::utils::opposite_player;

const ROWS: u8 = 6;
const COLS: u8 = 7;
const WIN_LENGTH: u8 = 4;

/// Represents the contents of a single Connect Four board cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum C4Cell {
    /// The cell is empty (no disc has been played here yet).
    Empty,
    /// The cell is occupied by Player1 (we'll treat this as 'Red').
    P1,
    /// The cell is occupied by Player2 (we'll treat this as 'Yellow').
    P2,
}

/// Represents a single move in Connect Four.
///
/// A move is "drop a disc into this column".
/// The row is determined by gravity (the lowest empty cell in that column).
#[derive(Clone, Copy, Debug)]
pub struct ConnectFourMove {
    /// The column index (0..=6) where the current player will drop a disc.
    pub column: u8,
}

/// Represents a full Connect Four game state.
///
/// The board is 7 columns by 6 rows, flattened into a 1D array of length 42.
/// We also track the current player and the height of each column
/// (i.e., how many cells are already filled in that column).
#[derive(Clone, Debug)]
pub struct ConnectFourState {
    /// The board cells, stored in row-major order, 6 rows × 7 columns = 42 cells.
    ///
    /// Indexing convention:
    ///   index = row * 7 + col
    /// where row in 0..6 (0 is top, 5 is bottom), col in 0..7.
    pub board: [C4Cell; 42],

    /// For each column (0..=6), how many discs have been placed in that column.
    ///
    /// This lets us quickly find the next empty row in a column when applying a move.
    /// heights[c] is in the range 0..=6. If heights[c] == 6, the column is full.
    pub heights: [u8; 7],

    /// The player whose turn it is to move in this position.
    pub current_player: Player,
}

impl Default for ConnectFourState {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectFourState {
    /// Creates a new Connect Four state representing the standard initial position:
    /// - all cells are empty,
    /// - all column heights are 0,
    /// - Player1 is to move.
    pub fn new() -> Self {
        Self {
            board: [C4Cell::Empty; 42],
            heights: [0; 7],
            current_player: Player::Player1,
        }
    }

    #[inline]
    fn idx(row: u8, col: u8) -> usize {
        (row as usize) * (COLS as usize) + col as usize
    }

    /// Returns:
    /// - Some(Player::Player1) if Player1 has a 4-in-a-row
    /// - Some(Player::Player2) if Player2 has a 4-in-a-row
    /// - None otherwise
    ///
    /// This does not check for draws; only for wins.
    fn winner(&self) -> Option<Player> {
        if let Some(p) = self.check_horizontal() {
            return Some(p);
        }
        if let Some(p) = self.check_vertical() {
            return Some(p);
        }
        if let Some(p) = self.check_diag_down() {
            return Some(p);
        }
        if let Some(p) = self.check_diag_up() {
            return Some(p);
        }
        None
    }

    /// Checks all horizontal lines for a 4-in-a-row.
    /// Returns the winning player if found.
    fn check_horizontal(&self) -> Option<Player> {
        for col in 0..=(COLS - WIN_LENGTH) {
            for row in 0..ROWS {
                let idxs = [
                    Self::idx(row, col),
                    Self::idx(row, col + 1),
                    Self::idx(row, col + 2),
                    Self::idx(row, col + 3),
                ];
                if let Some(p) = self.check_line(idxs) {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Checks vertical lines for 4-in-a-row.
    fn check_vertical(&self) -> Option<Player> {
        for col in 0..COLS {
            for row in 0..=(ROWS - WIN_LENGTH) {
                let idxs = [
                    Self::idx(row, col),
                    Self::idx(row + 1, col),
                    Self::idx(row + 2, col),
                    Self::idx(row + 3, col),
                ];
                if let Some(p) = self.check_line(idxs) {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Checks diagonal down-right lines (↘).
    fn check_diag_down(&self) -> Option<Player> {
        for col in 0..=(COLS - WIN_LENGTH) {
            for row in 0..=(ROWS - WIN_LENGTH) {
                let idxs = [
                    Self::idx(row, col),
                    Self::idx(row + 1, col + 1),
                    Self::idx(row + 2, col + 2),
                    Self::idx(row + 3, col + 3),
                ];
                if let Some(p) = self.check_line(idxs) {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Checks diagonal up-right lines (↗).
    fn check_diag_up(&self) -> Option<Player> {
        for col in 0..=(COLS - WIN_LENGTH) {
            for row in (WIN_LENGTH - 1)..ROWS {
                let idxs = [
                    Self::idx(row, col),
                    Self::idx(row - 1, col + 1),
                    Self::idx(row - 2, col + 2),
                    Self::idx(row - 3, col + 3),
                ];
                if let Some(p) = self.check_line(idxs) {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Given four board indices, returns Some(Player) if all four cells belong
    /// to the same player (and are not Empty), otherwise None.
    fn check_line(&self, idxs: [usize; 4]) -> Option<Player> {
        let is_p1 = idxs.iter().all(|&i| self.board[i] == C4Cell::P1);
        if is_p1 {
            return Some(Player::Player1);
        }
        let is_p2 = idxs.iter().all(|&i| self.board[i] == C4Cell::P2);
        if is_p2 {
            return Some(Player::Player2);
        }
        None
    }

    /// Attempts to construct a ConnectFourState from a string representation.
    ///
    /// Suggested format (you can adjust if you prefer):
    /// - A string of exactly 42 characters.
    /// - Each character represents a cell in row-major order.
    /// - Use:
    ///   - '.' for Empty,
    ///   - 'X' for Player1 (P1),
    ///   - 'O' for Player2 (P2).
    ///
    /// Example (one row per line for clarity, but string has no newlines):
    ///   ".......\
    ///    .......\
    ///    .......\
    ///    .......\
    ///    .......\
    ///    ......."
    ///
    /// You may ignore gravity correctness here for now; later, we can add validation
    /// that the board is "physically legal" (no discs floating above empties).
    pub fn from_str(repr: &str, current_player: Player) -> Result<Self, String> {
        // TODO: parse repr into board + heights + current_player.
        unimplemented!()
    }
}

impl GameState for ConnectFourState {
    /// The move type for Connect 4 is just a column index (0..=6).
    type Move = ConnectFourMove;

    /// Returns legal moves based on the current state. Legal moves are any column that
    /// is not already full.
    fn legal_moves(&self) -> Vec<Self::Move> {
        self.heights
            .iter()
            .enumerate()
            .filter_map(|(col, height)| {
                if *height < ROWS {
                    Some(Self::Move { column: col as u8 })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Applies a move by the current player and returns the resulting new state.
    ///
    /// The move is "drop a disc into column mv.column", where the disc falls
    /// to the lowest empty row in that column (determined by self.heights).
    ///
    /// This method:
    /// - computes the correct row via gravity,
    /// - fills in the appropriate C4Cell,
    /// - increments the column height,
    /// - switches the current player,
    /// - and returns the new ConnectFourState.
    ///
    /// This function must not mutate `self`; it returns a fresh state.
    fn apply_move(&self, mv: &Self::Move) -> Self {
        let mut new_board = self.board;
        let mut new_heights = self.heights;
        let col = mv.column;
        let row = (ROWS - 1) - self.heights[col as usize];
        let idx = Self::idx(row, col);

        new_board[idx] = if self.current_player == Player::Player1 {
            C4Cell::P1
        } else {
            C4Cell::P2
        };
        new_heights[col as usize] += 1;
        let new_player = opposite_player(self.current_player);
        Self {
            board: new_board,
            heights: new_heights,
            current_player: new_player,
        }
    }

    /// Return the current player.
    fn current_player(&self) -> Player {
        self.current_player
    }

    /// Determines if the game state is terminal.
    fn is_terminal(&self) -> bool {
        self.winner().is_some() || self.board.iter().all(|c| *c != C4Cell::Empty)
    }

    fn terminal_value(&self) -> Option<i32> {
        if let Some(p) = self.winner() {
            return Some(match p {
                Player::Player1 => 1,
                Player::Player2 => -1,
            });
        }

        if self.board.iter().all(|c| *c != C4Cell::Empty) {
            return Some(0);
        }

        None
    }
}

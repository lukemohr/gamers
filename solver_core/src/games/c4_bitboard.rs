use crate::game::{GameState, Player};
use crate::utils::opposite_player;

/// Board geometry for bitboard layout:
/// - 6 playable rows
/// - 7 columns
/// - 1 sentinel bit per column
const ROWS: u8 = 6;
const COLS: u8 = 7;
const BITS_PER_COL: u8 = ROWS + 1; // = 7
const WIN_LENGTH: u8 = 4;
const COL_WEIGHTS: [i32; 7] = [3, 4, 5, 7, 5, 4, 3];

/// Efficient bitboard-based representation of Connect Four.
///
/// Uses the canonical 7x6+padding layout:
/// - each column occupies 7 bits (6 playable + 1 sentinel)
/// - bit index = col * 7 + row
///
/// player_bb: bits for Player1's discs
/// mask_bb:  bits for all discs (P1 + P2)
/// heights: next free bit index for each column  
#[derive(Clone, Debug)]
pub struct BitboardState {
    pub player_bb: u64,
    pub mask_bb: u64,
    pub heights: [u8; COLS as usize],
    pub current_player: Player,
}

impl Default for BitboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl BitboardState {
    /// Creates a new empty bitboard state.
    pub fn new() -> Self {
        Self {
            player_bb: 0,
            mask_bb: 0,
            heights: [0; COLS as usize],
            current_player: Player::Player1,
        }
    }

    /// Computes the bit mask for the next empty cell in the given column.
    pub fn next_bit(&self, col: u8) -> u64 {
        let bit_index = (col * BITS_PER_COL) + self.heights[col as usize];
        1u64 << (bit_index as u64)
    }

    #[inline]
    fn idx(row: u8, col: u8) -> u8 {
        col * BITS_PER_COL + row
    }

    /// Applies a move in the given column and returns the resulting new state.
    ///
    /// This:
    /// - sets the bit for the current player
    /// - updates the mask_bb
    /// - increments heights[col]
    /// - swaps the current player
    pub fn apply_column_move(&self, col: u8) -> Self {
        let next_move = self.next_bit(col);
        let new_mask_bb = self.mask_bb | next_move;
        let mut new_heights = self.heights;
        new_heights[col as usize] += 1;
        let new_player_bb = match self.current_player {
            Player::Player1 => self.player_bb | next_move,
            Player::Player2 => self.player_bb,
        };
        let new_player = opposite_player(self.current_player);
        Self {
            player_bb: new_player_bb,
            mask_bb: new_mask_bb,
            heights: new_heights,
            current_player: new_player,
        }
    }

    #[inline]
    fn p2_bb(&self) -> u64 {
        self.mask_bb ^ self.player_bb
    }

    #[inline]
    fn has_run(bb: u64, shift: u8) -> bool {
        let s = shift as u64;
        let x = bb & (bb >> s);
        (x & (x >> (2 * s))) != 0
    }

    /// Checks whether the bitboard `bb` contains a 4-in-a-row.
    ///
    /// Win directions are detected using bit shifts:
    /// - Horizontal: shift by 1
    /// - Vertical:   shift by BITS_PER_COL (7)
    /// - Diag ↘:     shift by BITS_PER_COL + 1 (8)
    /// - Diag ↗:     shift by BITS_PER_COL - 1 (6)
    ///
    /// Returns true if any direction yields 4 aligned bits.
    pub fn check_win(&self, bb: u64) -> bool {
        Self::has_run(bb, 1)
            || Self::has_run(bb, BITS_PER_COL)
            || Self::has_run(bb, BITS_PER_COL + 1)
            || Self::has_run(bb, BITS_PER_COL - 1)
    }

    /// Returns true if the board is full (mask_bb contains all playable cells).
    pub fn is_full(&self) -> bool {
        self.heights.iter().all(|&h| h == ROWS)
    }

    /// Computes a heuristic evaluation based on:
    /// - all horizontal, vertical, and diagonal windows of length 4
    /// - center column occupancy
    pub fn evaluate(&self) -> i32 {
        let p1_board = self.player_bb;
        let p2_board = self.p2_bb();
        if let Some(v) = self.terminal_value() {
            // Scale terminal values so they dominate heuristic noise
            return v * 1000000;
        }
        self.score_all_windows(p1_board, p2_board) + self.center_control_score(p1_board, p2_board)
    }

    /// Scores all windows of 4 cells on the board.
    ///
    /// This function iterates over all possible 4-cell segments (horiz, vert, diag)
    /// and aggregates their contributions to the heuristic.
    fn score_all_windows(&self, p1_board: u64, p2_board: u64) -> i32 {
        self.check_horizontal(p1_board, p2_board)
            + self.check_vertical(p1_board, p2_board)
            + self.check_diag_down(p1_board, p2_board)
            + self.check_diag_up(p1_board, p2_board)
    }

    #[inline]
    fn window_mask(coords: &[(u8, u8); 4]) -> u64 {
        coords
            .iter()
            .fold(0u64, |acc, &(r, c)| acc | (1u64 << Self::idx(r, c) as u64))
    }

    /// Checks all horizontal lines for a 4-in-a-row.
    /// Returns the heuristic score.
    fn check_horizontal(&self, p1_board: u64, p2_board: u64) -> i32 {
        let mut score: i32 = 0;
        for col in 0..=(COLS - WIN_LENGTH) {
            for row in 0..ROWS {
                let coords = [(row, col), (row, col + 1), (row, col + 2), (row, col + 3)];
                let mask = Self::window_mask(&coords);
                score += self.score_window(p1_board, p2_board, mask);
            }
        }
        score
    }

    /// Checks vertical lines for 4-in-a-row.
    fn check_vertical(&self, p1_board: u64, p2_board: u64) -> i32 {
        let mut score: i32 = 0;
        for col in 0..COLS {
            for row in 0..=(ROWS - WIN_LENGTH) {
                let coords = [(row, col), (row + 1, col), (row + 2, col), (row + 3, col)];
                let mask = Self::window_mask(&coords);
                score += self.score_window(p1_board, p2_board, mask);
            }
        }
        score
    }

    /// Checks diagonal down-right lines (↘).
    fn check_diag_down(&self, p1_board: u64, p2_board: u64) -> i32 {
        let mut score: i32 = 0;
        for col in 0..=(COLS - WIN_LENGTH) {
            for row in 0..=(ROWS - WIN_LENGTH) {
                let coords = [
                    (row, col),
                    (row + 1, col + 1),
                    (row + 2, col + 2),
                    (row + 3, col + 3),
                ];
                let mask = Self::window_mask(&coords);
                score += self.score_window(p1_board, p2_board, mask);
            }
        }
        score
    }

    /// Checks diagonal up-right lines (↗).
    fn check_diag_up(&self, p1_board: u64, p2_board: u64) -> i32 {
        let mut score: i32 = 0;
        for col in 0..=(COLS - WIN_LENGTH) {
            for row in (WIN_LENGTH - 1)..ROWS {
                let coords = [
                    (row, col),
                    (row - 1, col + 1),
                    (row - 2, col + 2),
                    (row - 3, col + 3),
                ];
                let mask = Self::window_mask(&coords);
                score += self.score_window(p1_board, p2_board, mask);
            }
        }
        score
    }

    fn count_player_chips(&self, board: u64, mask: u64) -> u32 {
        (mask & board).count_ones()
    }

    /// Scores a single 4-cell window given as a mask (bitboard) of those 4 cells.
    ///
    /// `window_mask` selects the 4 cells.
    /// This method counts how many belong to Player1, how many to Player2,
    /// and returns a signed score contribution.
    fn score_window(&self, p1_board: u64, p2_board: u64, window_mask: u64) -> i32 {
        let num_p1_chips = self.count_player_chips(p1_board, window_mask);
        let num_p2_chips = self.count_player_chips(p2_board, window_mask);
        match (num_p1_chips, num_p2_chips) {
            (4, 0) => 100000,
            (3, 0) => 100,
            (2, 0) => 10,
            (0, 2) => -10,
            (0, 3) => -100,
            (0, 4) => -100000,
            _ => 0,
        }
    }

    fn score_column(&self, p1_board: u64, p2_board: u64, column: u8) -> i32 {
        let mut col_mask = 0u64;
        for row in 0..ROWS {
            let idx = Self::idx(row, column);
            col_mask |= 1u64 << idx as u64;
        }
        let num_p1_chips = self.count_player_chips(p1_board, col_mask) as i32;
        let num_p2_chips = self.count_player_chips(p2_board, col_mask) as i32;
        let w = COL_WEIGHTS[column as usize];
        w * (num_p1_chips - num_p2_chips)
    }

    /// Returns a bonus score for occupying central columns.
    ///
    /// A common heuristic is:
    /// - central column (col 3) is best
    /// - near-center columns (2,4) next
    /// - then (1,5)
    /// - then outer (0,6)
    fn center_control_score(&self, p1_board: u64, p2_board: u64) -> i32 {
        (0..COLS)
            .map(|col| self.score_column(p1_board, p2_board, col))
            .sum()
    }

    /// Returns a priority score for exploring a move (column) earlier in search.
    ///
    /// Semantics:
    /// - Higher values = more promising for the *current player* in this state.
    pub fn move_ordering_key_connect4(&self, col: u8) -> i32 {
        let bit = self.next_bit(col);

        let p1 = self.player_bb;
        let p2 = self.p2_bb();

        // Board after the *current player* plays in this column,
        // and board after the *opponent* would play in this column.
        let (curr_after, opp_after) = match self.current_player {
            Player::Player1 => (p1 | bit, p2 | bit),
            Player::Player2 => (p2 | bit, p1 | bit),
        };

        if self.check_win(curr_after) {
            // Immediate win for side to move
            1_000_000
        } else if self.check_win(opp_after) {
            // Column is an immediate win for the opponent, so playing here blocks it
            10_000
        } else {
            // Otherwise prefer central columns
            COL_WEIGHTS[col as usize]
        }
    }
}

impl GameState for BitboardState {
    type Move = u8; // column index (0..=6)

    /// Return legal moves (any column that is not full).
    fn legal_moves(&self) -> Vec<Self::Move> {
        (0..COLS)
            .filter(|&c| self.heights[c as usize] < ROWS)
            .collect()
    }

    /// Applies a move by dropping a disc into the given column `mv`.
    ///
    /// This delegates to BitboardState::apply_move(col), which performs:
    /// - bit placement
    /// - mask update
    /// - player1 bitboard update (if needed)
    /// - height increment
    /// - current player swap
    ///
    /// Does not mutate `self`; returns a fresh state.
    fn apply_move(&self, mv: &Self::Move) -> Self {
        self.apply_column_move(*mv)
    }

    /// Return the current player.
    fn current_player(&self) -> Player {
        self.current_player
    }

    /// Check if a board is terminal by checking if player 1 has a win, player 2 has a win,
    /// or if there's a draw.
    fn is_terminal(&self) -> bool {
        self.check_win(self.player_bb) || self.check_win(self.p2_bb()) || self.is_full()
    }

    /// Returns the game-theoretic value of the position if terminal:
    ///
    /// - +1 if Player1 has won
    /// - -1 if Player2 has won
    /// -  0 if board is full (draw)
    /// - None if the game is not terminal
    ///
    /// Note: bitboards make win checking extremely fast.
    fn terminal_value(&self) -> Option<i32> {
        if self.check_win(self.player_bb) {
            Some(1)
        } else if self.check_win(self.p2_bb()) {
            Some(-1)
        } else if self.is_full() {
            Some(0)
        } else {
            None
        }
    }

    /// Determine the heuristic value for the game state.
    fn heuristic_value(&self) -> i32 {
        self.evaluate()
    }

    fn move_ordering_key(&self, mv: &Self::Move) -> i32 {
        self.move_ordering_key_connect4(*mv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play_sequence(cols: &[u8]) -> BitboardState {
        let mut s = BitboardState::new();
        for &c in cols {
            s = s.apply_column_move(c);
        }
        s
    }

    #[test]
    fn c4_next_bit_matches_heights() {
        let s = BitboardState::new();
        // At start, heights are 0 so bit index for col 0 is 0.
        let bit0 = s.next_bit(0);
        assert_eq!(bit0, 1u64 << 0);

        // After one move in column 0, height becomes 1, next bit is 1 << 1.
        let s2 = s.apply_column_move(0);
        let bit1 = s2.next_bit(0);
        assert_eq!(bit1, 1u64 << 1);
    }

    #[test]
    fn c4_horizontal_win_detection_for_player1() {
        // P1: 0,1,2,3 (bottom row)
        // P2: dumps in column 6 to alternate turns
        let s = play_sequence(&[0, 6, 1, 6, 2, 6, 3]);
        assert!(s.check_win(s.player_bb)); // P1 should have a horizontal 4
        assert!(s.is_terminal());
        assert_eq!(s.terminal_value(), Some(1));
    }

    #[test]
    fn c4_vertical_win_detection_for_player1() {
        // P1: 0,0,0,0 (stacked)
        // P2: plays elsewhere to alternate
        let s = play_sequence(&[0, 6, 0, 6, 0, 6, 0]);
        assert!(s.check_win(s.player_bb));
        assert!(s.is_terminal());
        assert_eq!(s.terminal_value(), Some(1));
    }

    #[test]
    fn c4_diagonal_down_right_win_detection_for_player1() {
        // Build a ↘ diagonal for P1 starting at (row 0, col 0).
        //
        // A standard construction:
        // col 0: P1, P2, P2
        // col 1: P1, P2
        // col 2: P1
        // col 3: P1
        //
        // Sequence chosen to respect gravity and turn order.
        let s = play_sequence(&[
            0, 1, // P1:0, P2:1
            1, 2, // P1:1, P2:2
            2, 3, // P1:2, P2:3
            2, 3, // P1:2, P2:3
            3, 0, // P1:3, P2:0
            3, // P1:3 -> now P1 should have a diagonal
        ]);
        assert!(s.check_win(s.player_bb));
        assert!(s.is_terminal());
        assert_eq!(s.terminal_value(), Some(1));
    }

    #[test]
    fn c4_is_full_when_all_columns_full() {
        let mut s = BitboardState::new();
        for col in 0..COLS {
            for _ in 0..ROWS {
                s = s.apply_column_move(col);
            }
        }
        assert!(s.is_full());
        assert!(s.terminal_value().is_some()); // full and terminal, value can be -1/0/1
    }

    #[test]
    fn c4_move_ordering_prefers_winning_move() {
        // Construct a state where P1 has three in a row on bottom row [0,1,2]
        // and column 3 wins immediately.
        let s = play_sequence(&[0, 6, 1, 6, 2, 6]); // P1 to move, can win on 3
        let win_key = s.move_ordering_key(&3u8);
        let other_key = s.move_ordering_key(&0u8);
        assert!(win_key > other_key);
    }

    #[test]
    fn c4_depth_limited_search_prefers_winning_move() {
        let s = play_sequence(&[0, 6, 1, 6, 2, 6]); // P1 can win on 3
        // Depth 1 is enough to see immediate win
        let (best_move, value) =
            crate::solvers::minimax::minimax_best_move_ab_depth(&s, 1).expect("should have moves");
        assert_eq!(best_move, 3u8);
        assert!(value > 0);
    }

    #[test]
    fn c4_heuristic_is_symmetric_for_empty_board() {
        let s = BitboardState::new();
        assert_eq!(s.heuristic_value(), 0);
    }
}

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
    fn idx(row: u8, col: u8) -> usize {
        (row as usize) * (COLS as usize) + col as usize
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
        let x = bb & (bb >> shift);
        (x & (x >> (2 * shift))) != 0
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
        self.score_all_windows() + self.center_control_score()
    }

    /// Scores all windows of 4 cells on the board.
    ///
    /// This function iterates over all possible 4-cell segments (horiz, vert, diag)
    /// and aggregates their contributions to the heuristic.
    fn score_all_windows(&self) -> i32 {
        self.check_horizontal()
            + self.check_vertical()
            + self.check_diag_down()
            + self.check_diag_up()
    }

    /// Checks all horizontal lines for a 4-in-a-row.
    /// Returns the winning player if found.
    fn check_horizontal(&self) -> i32 {
        let mut score: i32 = 0;
        for col in 0..=(COLS - WIN_LENGTH) {
            for row in 0..ROWS {
                let mask = (1 << Self::idx(row, col))
                    | (1 << Self::idx(row, col + 1))
                    | (1 << Self::idx(row, col + 2))
                    | (1 << Self::idx(row, col + 3));
                score += self.score_window(mask);
            }
        }
        score
    }

    /// Checks vertical lines for 4-in-a-row.
    fn check_vertical(&self) -> i32 {
        let mut score: i32 = 0;
        for col in 0..COLS {
            for row in 0..=(ROWS - WIN_LENGTH) {
                let mask = (1 << Self::idx(row, col))
                    | (1 << Self::idx(row + 1, col))
                    | (1 << Self::idx(row + 2, col))
                    | (1 << Self::idx(row + 3, col));
                score += self.score_window(mask);
            }
        }
        score
    }

    /// Checks diagonal down-right lines (↘).
    fn check_diag_down(&self) -> i32 {
        let mut score: i32 = 0;
        for col in 0..=(COLS - WIN_LENGTH) {
            for row in 0..=(ROWS - WIN_LENGTH) {
                let mask = (1 << Self::idx(row, col))
                    | (1 << Self::idx(row + 1, col + 1))
                    | (1 << Self::idx(row + 2, col + 2))
                    | (1 << Self::idx(row + 3, col + 3));
                score += self.score_window(mask);
            }
        }
        score
    }

    /// Checks diagonal up-right lines (↗).
    fn check_diag_up(&self) -> i32 {
        let mut score: i32 = 0;
        for col in 0..=(COLS - WIN_LENGTH) {
            for row in (WIN_LENGTH - 1)..ROWS {
                let mask = (1 << Self::idx(row, col))
                    | (1 << Self::idx(row - 1, col + 1))
                    | (1 << Self::idx(row - 2, col + 2))
                    | (1 << Self::idx(row - 3, col + 3));
                score += self.score_window(mask);
            }
        }
        score
    }

    fn count_p1_chips(&self, mask: u64) -> u32 {
        (mask & self.player_bb).count_ones()
    }

    fn count_p2_chips(&self, mask: u64) -> u32 {
        (mask & self.p2_bb()).count_ones()
    }

    /// Scores a single 4-cell window given as a mask (bitboard) of those 4 cells.
    ///
    /// `window_mask` selects the 4 cells.
    /// This method counts how many belong to Player1, how many to Player2,
    /// and returns a signed score contribution.
    fn score_window(&self, window_mask: u64) -> i32 {
        let num_p1_chips = self.count_p1_chips(window_mask);
        let num_p2_chips = self.count_p2_chips(window_mask);
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

    fn score_column(&self, column: u8, mut score: i32) -> i32 {
        let locs: Vec<u8> = (0..ROWS).collect();
        let this_col = locs
            .iter()
            .map(|row| column * BITS_PER_COL + row)
            .map(|idx| 1u64 << idx);
        let mut col_chips = 0_u64;
        for col in this_col {
            col_chips |= col;
        }
        let num_p1_chips = self.count_p1_chips(col_chips);
        let num_p2_chips = self.count_p2_chips(col_chips);
        score += 10_i32.pow(3 - (column.abs_diff(3) as u32)) * (num_p1_chips as i32);
        score -= 10_i32.pow(3 - (column.abs_diff(3) as u32)) * (num_p2_chips as i32);
        score
    }

    /// Returns a bonus score for occupying central columns.
    ///
    /// A common heuristic is:
    /// - central column (col 3) is best
    /// - near-center columns (2,4) next
    /// - then (1,5)
    /// - then outer (0,6)
    fn center_control_score(&self) -> i32 {
        let mut score: i32 = 0;
        for col in 0..COLS {
            score = self.score_column(col, score)
        }
        score
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
}

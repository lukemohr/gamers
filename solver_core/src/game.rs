/// Represents the players in a two-player deterministic game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    Player1,
    Player2,
}

/// A trait representing a deterministic, perfect-information,
/// two-player, turn-based game state.
pub trait GameState: Clone {
    /// The type used to represent legal moves in this game.
    type Move: Clone;

    /// Returns the player whose turn it is in this state.
    fn current_player(&self) -> Player;

    /// Returns a list of all legal moves from this state.
    fn legal_moves(&self) -> Vec<Self::Move>;

    /// Applies a move to the state and returns the resulting state.
    /// This should *not* mutate `self`; instead, return a new value.
    fn apply_move(&self, mv: &Self::Move) -> Self;

    /// Returns true if the state is terminal (win/loss/draw).
    fn is_terminal(&self) -> bool;

    /// Returns the utility value (from Player1’s perspective)
    /// if the state is terminal.
    ///
    /// Convention:
    /// - +1 = Player1 win
    /// -  0 = draw
    /// - -1 = Player1 loss
    ///
    /// If state is non-terminal, return None.
    fn terminal_value(&self) -> Option<i32>;

    /// Returns a heuristic evaluation of the position from Player1's perspective.
    ///
    /// By convention:
    /// - Large positive values = good for Player1
    /// - Large negative values = good for Player2
    /// - 0 = roughly equal
    ///
    /// Default implementation:
    /// - If the position is terminal, return its terminal value
    /// - Otherwise, return 0 (neutral)
    fn heuristic_value(&self) -> i32 {
        self.terminal_value().unwrap_or(0)
    }

    /// Returns a heuristic key for move ordering from the perspective of the
    /// player to move in this state. Higher values should be explored earlier.
    /// Default implementation returns 0 for all moves.
    #[allow(unused_variables)]
    fn move_ordering_key(&self, mv: &Self::Move) -> i32 {
        0
    }
}

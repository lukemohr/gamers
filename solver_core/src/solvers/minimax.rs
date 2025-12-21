use crate::game::{GameState, Player};

/// Computes the minimax value of a state from Player1's perspective.
///
/// This is the "value" of the position assuming both players play perfectly.
///
/// Returns:
/// - +1 if Player1 is winning
/// - -1 if Player1 is losing
/// -  0 if the position is a forced draw
///
/// This version only returns the value, not the best move.
pub fn minimax_value<G: GameState>(state: &G) -> i32 {
    if state.is_terminal() {
        return state.terminal_value().unwrap();
    }
    let mvs = state
        .legal_moves()
        .into_iter()
        .map(|mv| minimax_value(&state.apply_move(&mv)));
    match state.current_player() {
        Player::Player1 => mvs.max(),
        Player::Player2 => mvs.min(),
    }
    .unwrap()
}

/// Computes the best move and its minimax value.
///
/// Returns `None` if the state has no legal moves (e.g., terminal state).
///
/// This is often what a game agent needs: the recommended move AND
/// the evaluation score.
pub fn minimax_best_move<G: GameState>(state: &G) -> Option<(G::Move, i32)> {
    let moves = state.legal_moves();
    if moves.is_empty() {
        return None;
    }
    let maximizing = state.current_player() == Player::Player1;
    let mut best_value = if maximizing { i32::MIN } else { i32::MAX };
    let mut best_move = None;

    for mv in &moves {
        let child_value = minimax_value(&state.apply_move(mv));

        let is_better = if maximizing {
            child_value > best_value
        } else {
            child_value < best_value
        };

        if is_better {
            best_value = child_value;
            best_move = Some(mv.clone());
        }
    }

    best_move.map(|m| (m, best_value))
}

/// Computes the minimax value with alpha-beta pruning.
///
/// `alpha` is the best value that Player1 (maximizing player) has guaranteed so far.
/// `beta` is the best value that Player2 (minimizing player) has guaranteed so far.
///
/// Returns:
/// - +1, 0, or -1 depending on perfect play outcome.
///
/// IMPORTANT:
/// - This function must prune branches where `alpha >= beta`.
/// - This is the core recursive function; the user should usually call
///   `minimax_value_ab` instead.
pub fn minimax_value_ab<G: GameState>(state: &G, mut alpha: i32, mut beta: i32) -> i32 {
    if state.is_terminal() {
        return state.terminal_value().unwrap();
    }
    let maximizing = state.current_player() == Player::Player1;
    let mut value = if maximizing { i32::MIN } else { i32::MAX };
    let mut moves = state.legal_moves();

    // Higher move_ordering_key = more promising for the current player
    moves.sort_by_key(|m| std::cmp::Reverse(state.move_ordering_key(m)));
    for mv in &moves {
        let child_value = minimax_value_ab(&state.apply_move(mv), alpha, beta);

        if maximizing {
            value = value.max(child_value);
            alpha = alpha.max(value);
        } else {
            value = value.min(child_value);
            beta = beta.min(value);
        }

        if alpha >= beta {
            break;
        }
    }
    value
}

/// Computes the minimax value with alpha-beta pruning,
/// using the full range [-∞, +∞] as the initial bounds.
pub fn minimax_value_ab_root<G: GameState>(state: &G) -> i32 {
    minimax_value_ab(state, i32::MIN, i32::MAX)
}

fn minimax_best_move_ab_inner<G: GameState>(
    state: &G,
    mut alpha: i32,
    mut beta: i32,
) -> Option<(G::Move, i32)> {
    let mut moves = state.legal_moves();
    if moves.is_empty() {
        return None;
    }
    let maximizing = state.current_player() == Player::Player1;

    // Higher move_ordering_key = more promising for the current player
    moves.sort_by_key(|m| std::cmp::Reverse(state.move_ordering_key(m)));

    let mut best_value = if maximizing { i32::MIN } else { i32::MAX };
    let mut best_move = None;

    for mv in &moves {
        let child_value = minimax_value_ab(&state.apply_move(mv), alpha, beta);

        let is_better = if maximizing {
            child_value > best_value
        } else {
            child_value < best_value
        };

        if is_better {
            best_value = child_value;
            best_move = Some(mv.clone());
        }

        if maximizing {
            alpha = alpha.max(best_value);
        } else {
            beta = beta.min(best_value);
        }

        if alpha >= beta {
            break;
        }
    }

    best_move.map(|m| (m, best_value))
}

/// Computes the best move using alpha-beta pruning.
///
/// Returns:
/// - Some((best_move, value)) if a move exists.
/// - None if there are no legal moves (terminal state).
///
/// This should prune as much as possible during search.
pub fn minimax_best_move_ab<G: GameState>(state: &G) -> Option<(G::Move, i32)> {
    minimax_best_move_ab_inner(state, i32::MIN, i32::MAX)
}

/// Depth-limited alpha-beta minimax.
///
/// - `depth` = maximum remaining ply to search.
/// - Uses `state.heuristic_value()` when depth == 0 or at terminal states.
pub fn minimax_value_ab_depth<G: GameState>(
    state: &G,
    depth: u32,
    mut alpha: i32,
    mut beta: i32,
) -> i32 {
    if let Some(v) = state.terminal_value() {
        return v * 1_000_000;
    }

    // At depth 0, use the heuristic only (non-terminal states).
    if depth == 0 {
        return state.heuristic_value();
    }
    let maximizing = state.current_player() == Player::Player1;
    let mut value = if maximizing { i32::MIN } else { i32::MAX };
    let mut moves = state.legal_moves();

    // Higher move_ordering_key = more promising for the current player
    moves.sort_by_key(|m| std::cmp::Reverse(state.move_ordering_key(m)));

    for mv in &moves {
        let child_value = minimax_value_ab_depth(&state.apply_move(mv), depth - 1, alpha, beta);

        if maximizing {
            value = value.max(child_value);
            alpha = alpha.max(value);
        } else {
            value = value.min(child_value);
            beta = beta.min(value);
        }

        if alpha >= beta {
            break;
        }
    }
    value
}

/// Convenience wrapper using full [-∞, +∞] initial bounds.
pub fn minimax_value_ab_depth_root<G: GameState>(state: &G, depth: u32) -> i32 {
    minimax_value_ab_depth(state, depth, i32::MIN, i32::MAX)
}

/// Returns the best move and its value at the given search depth.
/// Uses depth-limited alpha-beta with heuristic cutoff.
pub fn minimax_best_move_ab_depth_inner<G: GameState>(
    state: &G,
    depth: u32,
    mut alpha: i32,
    mut beta: i32,
) -> Option<(G::Move, i32)> {
    let mut moves = state.legal_moves();
    if moves.is_empty() {
        return None;
    }
    let maximizing = state.current_player() == Player::Player1;

    // Higher move_ordering_key = more promising for the current player
    moves.sort_by_key(|m| std::cmp::Reverse(state.move_ordering_key(m)));

    let mut best_value = if maximizing { i32::MIN } else { i32::MAX };
    let mut best_move = None;

    for mv in &moves {
        let child_value = minimax_value_ab_depth(&state.apply_move(mv), depth - 1, alpha, beta);

        let is_better = if maximizing {
            child_value > best_value
        } else {
            child_value < best_value
        };

        if is_better {
            best_value = child_value;
            best_move = Some(mv.clone());
        }

        if maximizing {
            alpha = alpha.max(best_value);
        } else {
            beta = beta.min(best_value);
        }

        if alpha >= beta {
            break;
        }
    }

    best_move.map(|m| (m, best_value))
}

pub fn minimax_best_move_ab_depth<G: GameState>(state: &G, depth: u32) -> Option<(G::Move, i32)> {
    minimax_best_move_ab_depth_inner(state, depth, i32::MIN, i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::c4_bitboard::BitboardState;
    use crate::games::ttt::TicTacToeState;

    #[test]
    fn minimax_and_ab_agree_on_ttt_start() {
        let s = TicTacToeState::new();
        let v_plain = minimax_value(&s);
        let v_ab = minimax_value_ab_root(&s);
        assert_eq!(v_plain, v_ab);
    }

    #[test]
    fn minimax_best_move_returns_same_value_as_value_function() {
        let s = TicTacToeState::new();
        let v_plain = minimax_value(&s);
        let (_mv, v_best) = minimax_best_move(&s).expect("there should be legal moves");
        assert_eq!(v_plain, v_best);
    }

    #[test]
    fn depth_limited_search_matches_full_search_on_ttt_at_full_depth() {
        let s = TicTacToeState::new();
        let v_full = minimax_value_ab_root(&s); // -1, 0, or 1
        let v_depth = minimax_value_ab_depth_root(&s, 9); // -1e6, 0, or 1e6

        assert_eq!(v_depth, v_full * 1_000_000);
    }

    #[test]
    fn c4_depth_zero_uses_heuristic() {
        let s = BitboardState::new();
        let v0 = minimax_value_ab_depth_root(&s, 0);
        let v1 = minimax_value_ab_depth_root(&s, 1);

        assert_eq!(v0, s.heuristic_value());
        assert!(v1 >= v0);
    }
}

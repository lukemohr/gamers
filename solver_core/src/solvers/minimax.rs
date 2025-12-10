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
    // Sort descending for maximizing, ascending for minimizing
    if maximizing {
        moves.sort_by_key(|m| std::cmp::Reverse(state.move_ordering_key(m)));
    } else {
        moves.sort_by_key(|m| state.move_ordering_key(m));
    }
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

    // Sort descending for maximizing, ascending for minimizing
    if maximizing {
        moves.sort_by_key(|m| std::cmp::Reverse(state.move_ordering_key(m)));
    } else {
        moves.sort_by_key(|m| state.move_ordering_key(m));
    }

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

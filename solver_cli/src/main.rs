//! Command-line interface for the game solver engine.
//!
//! This binary crate:
//! - parses command-line arguments,
//! - constructs game states,
//! - calls solver_core methods,
//! - prints solver outputs.

use std::io::{self, Write};

use solver_core::game::{GameState, Player};
use solver_core::games::ttt::{TicTacToeState, parse_ttt_move, print_ttt_board};
use solver_core::solvers::minimax::minimax_best_move_ab_depth;

/// Plays a human-vs-AI game of Tic-Tac-Toe in the terminal.
///
/// - `human_is_player1`: if true, human plays X (Player1), else O (Player2).
/// - `ai_depth`: search depth to use for the AI (for TTT, 9 is "perfect").
pub fn play_ttt_human_vs_ai(human_is_player1: bool, ai_depth: u32) {
    let mut state = TicTacToeState::new();

    println!("Welcome to Tic-Tac-Toe!");
    println!(
        "You are {}.",
        if human_is_player1 {
            "X (Player1)"
        } else {
            "O (Player2)"
        }
    );
    println!("Index mapping:");
    println!("0 | 1 | 2");
    println!("3 | 4 | 5");
    println!("6 | 7 | 8");
    println!();

    // Main game loop
    loop {
        print_ttt_board(&state);
        println!();

        if state.is_terminal() {
            break;
        }

        let current = state.current_player();
        let human_turn = (current == Player::Player1 && human_is_player1)
            || (current == Player::Player2 && !human_is_player1);

        if human_turn {
            // Human move
            println!("Your turn ({:?}).", current);
            loop {
                print!("Enter your move (0-8): ");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err() {
                    println!("Failed to read line. Try again.");
                    continue;
                }

                match parse_ttt_move(&input, &state) {
                    Ok(mv) => {
                        state = state.apply_move(&mv);
                        break;
                    }
                    Err(msg) => {
                        println!("Invalid move: {msg}");
                        continue;
                    }
                }
            }
        } else {
            // AI move
            println!("AI ({:?}) is thinking...", current);

            if let Some((mv, value)) = minimax_best_move_ab_depth(&state, ai_depth) {
                println!("AI chooses index {} (value = {}).", mv.index, value);
                state = state.apply_move(&mv);
            } else {
                // No moves: should only happen if state is terminal
                println!("AI has no legal moves.");
                break;
            }
        }
    }

    // Game over: print final board and result
    print_ttt_board(&state);
    println!("\nGame over!");

    match state.terminal_value() {
        Some(1) => println!("Player1 (X) wins!"),
        Some(-1) => println!("Player2 (O) wins!"),
        Some(0) => println!("It's a draw!"),
        None => println!("Non-terminal state at end? (Bug)"),
        _ => unreachable!("Should only see 1, -1, or 0 values for TTT."),
    }
}

fn main() {
    // Human plays X, AI plays O at depth 9 (perfect play)
    play_ttt_human_vs_ai(true, 9);
}

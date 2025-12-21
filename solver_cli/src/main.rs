//! Command-line interface for the game solver engine.
//!
//! This binary crate:
//! - parses command-line arguments,
//! - constructs game states,
//! - calls solver_core methods,
//! - prints solver outputs.

use solver_core::game::{GameState, Player};
use solver_core::games::c4_bitboard::{BitboardState, parse_c4_move, print_c4_board_bitboard};
use solver_core::games::ttt::{TicTacToeState, parse_ttt_move, print_ttt_board};
use solver_core::solvers::minimax::minimax_best_move_ab_depth;
use std::io::{self, Write};

/// Which game the user wants to play in the CLI.
#[derive(Clone, Copy, Debug)]
enum GameChoice {
    TicTacToe,
    ConnectFour,
}

/// Prompts the user to choose a game to play.
///
/// Prints a small menu like:
///   1) Tic-Tac-Toe
///   2) Connect Four
///
/// Then reads a line from stdin and returns:
/// - Ok(GameChoice::TicTacToe) for "1"
/// - Ok(GameChoice::ConnectFour) for "2"
/// - Err(String) if the input is invalid.
fn prompt_game_choice() -> Result<GameChoice, String> {
    println!("Please select the game you want to play:");
    println!("1: Tic-Tac-Toe");
    println!("2: Connect Four");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_| "Failed to read line.".to_string())?;

    let clean = input.trim();
    match clean {
        "1" => Ok(GameChoice::TicTacToe),
        "2" => Ok(GameChoice::ConnectFour),
        _ => Err("Invalid Selection".to_string()),
    }
}

/// Asks the user whether they want to be Player1 (X) or Player2 (O).
fn prompt_human_is_player1() -> Result<bool, String> {
    println!("Do you want to be Player 1 (X)? [y/n]");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_| "Failed to read line.".to_string())?;

    let clean = input.trim();
    match clean {
        "y" | "Y" => Ok(true),
        "n" | "N" => Ok(false),
        _ => Err("Please answer y or n".to_string()),
    }
}

/// Prompts the user for the AI search depth.
///
/// `default_depth` is used if the user just hits Enter.
fn prompt_ai_depth(default_depth: u32) -> Result<u32, String> {
    println!("Enter AI search depth (press Enter for default = {default_depth}):");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_| "Failed to read line.".to_string())?;

    let clean = input.trim();
    if clean.is_empty() {
        Ok(default_depth)
    } else {
        clean
            .parse::<u32>()
            .map_err(|_| "Could not parse depth as u32".to_string())
    }
}

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
            let symbol = if current == Player::Player1 { 'X' } else { 'O' };
            println!("Your turn ({symbol}).");
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

/// Plays a human-vs-AI game of Connect Four (bitboard) in the terminal.
///
/// - `human_is_player1`: if true, human is Player1 (X), else Player2 (O).
/// - `ai_depth`: search depth used by the AI.
pub fn play_c4_human_vs_ai(human_is_player1: bool, ai_depth: u32) {
    let mut state = BitboardState::new();

    println!("Welcome to Connect 4!");
    println!(
        "You are {}.",
        if human_is_player1 {
            "X (Player1)"
        } else {
            "O (Player2)"
        }
    );
    println!();

    // Main game loop
    loop {
        print_c4_board_bitboard(&state);
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
                print!("Enter your column (0-6): ");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err() {
                    println!("Failed to read line. Try again.");
                    continue;
                }

                match parse_c4_move(&input, &state) {
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
                println!("AI chooses column {} (value = {}).", mv, value);
                state = state.apply_move(&mv);
            } else {
                // No moves: should only happen if state is terminal
                println!("AI has no legal moves.");
                break;
            }
        }
    }

    // Game over: print final board and result
    print_c4_board_bitboard(&state);
    println!("\nGame over!");

    match state.terminal_value() {
        Some(1) => println!("Player1 (X) wins!"),
        Some(-1) => println!("Player2 (O) wins!"),
        Some(0) => println!("It's a draw!"),
        None => println!("Non-terminal state at end? (Bug)"),
        _ => unreachable!("Unreachable."),
    }
}

fn main() {
    println!("Game Solver CLI");
    println!("================\n");

    // 1) Choose game
    let game_choice = loop {
        match prompt_game_choice() {
            Ok(choice) => break choice,
            Err(msg) => {
                println!("Error: {msg}");
                continue;
            }
        }
    };

    // 2) Choose side
    let human_is_player1 = loop {
        match prompt_human_is_player1() {
            Ok(b) => break b,
            Err(msg) => {
                println!("Error: {msg}");
                continue;
            }
        }
    };

    // 3) Choose depth (maybe different defaults per game later)
    let ai_depth = loop {
        match prompt_ai_depth(9) {
            Ok(d) => break d,
            Err(msg) => {
                println!("Error: {msg}");
                continue;
            }
        }
    };

    // 4) Dispatch to the appropriate game loop
    match game_choice {
        GameChoice::TicTacToe => play_ttt_human_vs_ai(human_is_player1, ai_depth),
        GameChoice::ConnectFour => play_c4_human_vs_ai(human_is_player1, ai_depth),
    }
}

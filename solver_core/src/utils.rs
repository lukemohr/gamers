use crate::game::Player;

/// Returns the opponent of the given player.
pub fn opposite_player(p: Player) -> Player {
    match p {
        Player::Player1 => Player::Player2,
        Player::Player2 => Player::Player1,
    }
}

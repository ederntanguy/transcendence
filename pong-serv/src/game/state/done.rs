use crate::game::Side;

/// Result of a game, with a winner, a score and the reason for the victory.
pub struct GameResult {
    pub score: [u32; 2],
    pub winner: Side,
    pub win_type: WinType,
}

impl GameResult {
    pub(super) fn new(score: [u32; 2], winner: Side, win_type: WinType) -> Self {
        Self {
            score,
            winner,
            win_type,
        }
    }
}

/// Reason for the victory of a player.
pub enum WinType {
    /// The winner reached the number of points tha make him win the game according to the rules.
    ScoreReached,

    /// The winner's opponent disconnected, effectively withdrawing.
    Withdrawal,
}

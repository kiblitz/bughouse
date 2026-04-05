use shakmaty::fen::{Fen, LossyFenError};
use shakmaty::uci::UciMove;
use shakmaty::variant::Crazyhouse;
use shakmaty::{CastlingMode, EnPassantMode, FromSetup, Position};

use crate::game::types::{Color, GameMove, Reserve, Role};

/// Wraps a single Crazyhouse board with helper methods for bughouse.
///
/// In bughouse, each board plays by Crazyhouse rules (pieces can be dropped
/// from a reserve/pocket). The coordination of piece transfers between boards
/// is handled by `BughouseGame`, not here.
#[derive(Debug, Clone)]
pub struct Board {
    position: Crazyhouse,
}

/// Error returned when a move is illegal.
#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error("illegal move: {0}")]
    Illegal(String),
    #[error("invalid square: {0}")]
    InvalidSquare(String),
    #[error("not your turn")]
    NotYourTurn,
    #[error("game is over")]
    GameOver,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    /// Creates a new board at the standard starting position.
    pub fn new() -> Self {
        Self {
            position: Crazyhouse::new(),
        }
    }

    /// Returns whose turn it is.
    pub fn turn(&self) -> Color {
        self.position.turn().into()
    }

    /// Returns the FEN string of the current position.
    pub fn fen(&self) -> String {
        let setup = self.position.to_setup(EnPassantMode::Legal);
        Fen::try_from_setup(setup)
            .unwrap_or_else(LossyFenError::ignore)
            .to_string()
    }

    /// Returns true if the position is checkmate.
    pub fn is_checkmate(&self) -> bool {
        self.position.is_checkmate()
    }

    /// Returns true if the position is stalemate.
    pub fn is_stalemate(&self) -> bool {
        self.position.is_stalemate()
    }

    /// Returns true if the game is over (checkmate or stalemate).
    pub fn is_game_over(&self) -> bool {
        self.position.is_game_over()
    }

    /// Attempts to make a move. Returns the role of the captured piece, if any.
    ///
    /// Note: In standard Crazyhouse, captured pieces go to the capturer's pocket.
    /// For bughouse, the caller (`BughouseGame`) intercepts the capture and routes
    /// it to the partner's pocket on the other board instead.
    pub fn make_move(&mut self, game_move: &GameMove) -> Result<Option<Role>, MoveError> {
        let uci = self.game_move_to_uci(game_move)?;
        let m = uci
            .to_move(&self.position)
            .map_err(|e| MoveError::Illegal(e.to_string()))?;

        // Check what piece is on the target square before the move (for captures).
        let captured = m.capture().map(Role::from);

        // Before playing, snapshot the pocket state so we can undo crazyhouse's
        // automatic pocket addition.
        let pocket_before = self.position.pockets().cloned();

        self.position.play_unchecked(m);

        // Crazyhouse automatically adds captured pieces to the capturer's pocket.
        // For bughouse, we need to undo this — the piece goes to the partner's
        // pocket on the other board. Restore the pocket to pre-move state.
        if let (Some(_), Some(before)) = (captured, pocket_before) {
            let mut setup = self.position.to_setup(EnPassantMode::Legal);
            setup.pockets = Some(before);
            self.position = Crazyhouse::from_setup(setup, CastlingMode::Standard)
                .or_else(|e| e.ignore_too_much_material())
                .expect("position after move with restored pockets should be valid");
        }

        Ok(captured)
    }

    /// Adds a piece to the specified color's pocket on this board.
    /// Used when the partner captures a piece on the other board.
    pub fn add_to_pocket(&mut self, color: Color, role: Role) {
        let mut setup = self.position.to_setup(EnPassantMode::Legal);
        if let Some(ref mut pockets) = setup.pockets {
            let shak_color: shakmaty::Color = color.into();
            let shak_role: shakmaty::Role = role.into();
            pockets[shak_color][shak_role] += 1;
        }
        // In bughouse, pockets can exceed normal material limits since pieces
        // transfer between boards, so we must ignore the material check.
        self.position = Crazyhouse::from_setup(setup, CastlingMode::Standard)
            .or_else(|e| e.ignore_too_much_material())
            .expect("adding to pocket should produce valid position");
    }

    /// Returns the current reserve (pocket) for the given color.
    pub fn reserve(&self, color: Color) -> Reserve {
        let shak_color: shakmaty::Color = color.into();
        match self.position.pockets() {
            Some(pockets) => {
                let pocket = &pockets[shak_color];
                Reserve {
                    pawn: pocket[shakmaty::Role::Pawn],
                    knight: pocket[shakmaty::Role::Knight],
                    bishop: pocket[shakmaty::Role::Bishop],
                    rook: pocket[shakmaty::Role::Rook],
                    queen: pocket[shakmaty::Role::Queen],
                }
            }
            None => Reserve::default(),
        }
    }

    fn game_move_to_uci(&self, game_move: &GameMove) -> Result<UciMove, MoveError> {
        match game_move {
            GameMove::Normal {
                from,
                to,
                promotion,
            } => {
                let from_sq = parse_square(from)?;
                let to_sq = parse_square(to)?;
                let promo = promotion.map(shakmaty::Role::from);
                Ok(UciMove::Normal {
                    from: from_sq,
                    to: to_sq,
                    promotion: promo,
                })
            }
            GameMove::Drop { role, to } => {
                let to_sq = parse_square(to)?;
                let shak_role: shakmaty::Role = (*role).into();
                Ok(UciMove::Put {
                    role: shak_role,
                    to: to_sq,
                })
            }
        }
    }
}

fn parse_square(s: &str) -> Result<shakmaty::Square, MoveError> {
    s.parse::<shakmaty::Square>()
        .map_err(|_| MoveError::InvalidSquare(s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_board_starts_at_initial_position() {
        let board = Board::new();
        assert_eq!(board.turn(), Color::White);
        assert!(!board.is_game_over());
    }

    #[test]
    fn can_make_e4() {
        let mut board = Board::new();
        let m = GameMove::Normal {
            from: "e2".to_string(),
            to: "e4".to_string(),
            promotion: None,
        };
        let captured = board.make_move(&m).unwrap();
        assert!(captured.is_none());
        assert_eq!(board.turn(), Color::Black);
    }

    #[test]
    fn illegal_move_is_rejected() {
        let mut board = Board::new();
        let m = GameMove::Normal {
            from: "e2".to_string(),
            to: "e5".to_string(), // Can't move pawn 3 squares
            promotion: None,
        };
        assert!(board.make_move(&m).is_err());
    }

    #[test]
    fn capture_returns_captured_role() {
        let mut board = Board::new();
        // 1. e4 d5 2. exd5
        let moves = vec![
            GameMove::Normal {
                from: "e2".to_string(),
                to: "e4".to_string(),
                promotion: None,
            },
            GameMove::Normal {
                from: "d7".to_string(),
                to: "d5".to_string(),
                promotion: None,
            },
            GameMove::Normal {
                from: "e4".to_string(),
                to: "d5".to_string(),
                promotion: None,
            },
        ];
        for m in &moves[..2] {
            board.make_move(m).unwrap();
        }
        let captured = board.make_move(&moves[2]).unwrap();
        assert_eq!(captured, Some(Role::Pawn));

        // In bughouse mode, the pocket should NOT have the captured piece
        // (we restored the pre-capture pocket state).
        let white_reserve = board.reserve(Color::White);
        assert_eq!(white_reserve.pawn, 0);
    }

    #[test]
    fn add_to_pocket_and_drop() {
        let mut board = Board::new();
        // Make moves so it's white's turn again, then test drop
        board
            .make_move(&GameMove::Normal {
                from: "e2".to_string(),
                to: "e4".to_string(),
                promotion: None,
            })
            .unwrap();
        board
            .make_move(&GameMove::Normal {
                from: "e7".to_string(),
                to: "e5".to_string(),
                promotion: None,
            })
            .unwrap();

        // Add a knight to white's pocket
        board.add_to_pocket(Color::White, Role::Knight);
        let reserve = board.reserve(Color::White);
        assert_eq!(reserve.knight, 1);

        // Drop the knight on d3
        let drop = GameMove::Drop {
            role: Role::Knight,
            to: "d3".to_string(),
        };
        let captured = board.make_move(&drop).unwrap();
        assert!(captured.is_none());
        let reserve = board.reserve(Color::White);
        assert_eq!(reserve.knight, 0);
    }
}

use serde::{Deserialize, Serialize};

/// Identifies which board in a bughouse game.
/// Used in Phase 2+ for two-board bughouse coordination.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BoardId {
    A,
    B,
}

/// Chess piece color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn other(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl From<shakmaty::Color> for Color {
    fn from(c: shakmaty::Color) -> Self {
        match c {
            shakmaty::Color::White => Color::White,
            shakmaty::Color::Black => Color::Black,
        }
    }
}

impl From<Color> for shakmaty::Color {
    fn from(c: Color) -> Self {
        match c {
            Color::White => shakmaty::Color::White,
            Color::Black => shakmaty::Color::Black,
        }
    }
}

/// Chess piece role (type without color).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl From<shakmaty::Role> for Role {
    fn from(r: shakmaty::Role) -> Self {
        match r {
            shakmaty::Role::Pawn => Role::Pawn,
            shakmaty::Role::Knight => Role::Knight,
            shakmaty::Role::Bishop => Role::Bishop,
            shakmaty::Role::Rook => Role::Rook,
            shakmaty::Role::Queen => Role::Queen,
            shakmaty::Role::King => Role::King,
        }
    }
}

impl From<Role> for shakmaty::Role {
    fn from(r: Role) -> Self {
        match r {
            Role::Pawn => shakmaty::Role::Pawn,
            Role::Knight => shakmaty::Role::Knight,
            Role::Bishop => shakmaty::Role::Bishop,
            Role::Rook => shakmaty::Role::Rook,
            Role::Queen => shakmaty::Role::Queen,
            Role::King => shakmaty::Role::King,
        }
    }
}

/// A square on the chess board (e.g. "e4").
pub type Square = String;

/// A chess move: either a board move or a piece drop.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameMove {
    #[serde(rename = "move")]
    Normal {
        from: Square,
        to: Square,
        #[serde(skip_serializing_if = "Option::is_none")]
        promotion: Option<Role>,
    },
    #[serde(rename = "drop")]
    Drop { role: Role, to: Square },
}

/// Piece counts in a player's reserve (pocket).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Reserve {
    pub pawn: u8,
    pub knight: u8,
    pub bishop: u8,
    pub rook: u8,
    pub queen: u8,
}

impl Reserve {
    pub fn get(&self, role: Role) -> u8 {
        match role {
            Role::Pawn => self.pawn,
            Role::Knight => self.knight,
            Role::Bishop => self.bishop,
            Role::Rook => self.rook,
            Role::Queen => self.queen,
            Role::King => 0, // Kings are never in reserve
        }
    }

    pub fn add(&mut self, role: Role) {
        match role {
            Role::Pawn => self.pawn += 1,
            Role::Knight => self.knight += 1,
            Role::Bishop => self.bishop += 1,
            Role::Rook => self.rook += 1,
            Role::Queen => self.queen += 1,
            Role::King => {} // Kings cannot be captured in bughouse
        }
    }

    pub fn remove(&mut self, role: Role) -> bool {
        let count = match role {
            Role::Pawn => &mut self.pawn,
            Role::Knight => &mut self.knight,
            Role::Bishop => &mut self.bishop,
            Role::Rook => &mut self.rook,
            Role::Queen => &mut self.queen,
            Role::King => return false,
        };
        if *count > 0 {
            *count -= 1;
            true
        } else {
            false
        }
    }
}

/// The result of a completed game.
///
/// In Phase 1 (single board), `winner` is the color that won.
/// In Phase 2+ (bughouse), this will be extended with team semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameResult {
    /// The color that won, or None for a draw.
    pub winner: Option<Color>,
    pub termination: Termination,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Termination {
    Checkmate,
    Timeout,
    Resignation,
    Abandoned,
    Draw,
}

/// Current phase of a bughouse game. Used in Phase 2+.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamePhase {
    /// Waiting for players to ready up.
    Waiting,
    /// Game is actively being played.
    Active,
    /// Game has ended.
    Finished(GameResult),
}

/// Identifies a player's seat in a bughouse game. Used in Phase 2+.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Seat {
    pub board: BoardId,
    pub color: Color,
}

#[allow(dead_code)]
impl Seat {
    /// Returns the partner seat (same team, other board).
    pub fn partner(self) -> Self {
        Seat {
            board: match self.board {
                BoardId::A => BoardId::B,
                BoardId::B => BoardId::A,
            },
            color: self.color.other(),
        }
    }

    /// Returns the team number (1 or 2).
    /// Team 1: Board A White + Board B Black
    /// Team 2: Board A Black + Board B White
    pub fn team(self) -> u8 {
        match (self.board, self.color) {
            (BoardId::A, Color::White) | (BoardId::B, Color::Black) => 1,
            (BoardId::A, Color::Black) | (BoardId::B, Color::White) => 2,
        }
    }
}

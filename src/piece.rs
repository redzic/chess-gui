use std::ops::Not;

// x-offsets for displaying
pub const PAWN_XOFF: i32 = 10;
pub const KNIGHT_XOFF: i32 = 5;
pub const ROOK_XOFF: i32 = 6;
pub const BISHOP_XOFF: i32 = 3;
pub const QUEEN_XOFF: i32 = -2;
pub const KING_XOFF: i32 = 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceColor {
  White = 0,
  Black = 1,
}

impl Not for PieceColor {
  type Output = Self;

  fn not(self) -> Self::Output {
    // correctly compiles as (self ^ 1)
    match self {
      PieceColor::White => PieceColor::Black,
      PieceColor::Black => PieceColor::White,
    }
  }
}

impl PieceColor {
  #[inline]
  pub fn is_white(self) -> bool {
    self == PieceColor::White
  }

  #[inline]
  pub fn is_black(self) -> bool {
    self == PieceColor::Black
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceType {
  Pawn = 0,
  Knight = 1,
  Bishop = 2,
  Rook = 3,
  Queen = 4,
  King = 5,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Piece {
  pub class: PieceType,
  pub color: PieceColor,
}

pub const WP: Piece = Piece {
  class: PieceType::Pawn,
  color: PieceColor::White,
};
pub const BP: Piece = Piece {
  class: PieceType::Pawn,
  color: PieceColor::Black,
};

pub const WN: Piece = Piece {
  class: PieceType::Knight,
  color: PieceColor::White,
};
pub const BN: Piece = Piece {
  class: PieceType::Knight,
  color: PieceColor::Black,
};

pub const WB: Piece = Piece {
  class: PieceType::Bishop,
  color: PieceColor::White,
};
pub const BB: Piece = Piece {
  class: PieceType::Bishop,
  color: PieceColor::Black,
};

pub const WR: Piece = Piece {
  class: PieceType::Rook,
  color: PieceColor::White,
};
pub const BR: Piece = Piece {
  class: PieceType::Rook,
  color: PieceColor::Black,
};

pub const WQ: Piece = Piece {
  class: PieceType::Queen,
  color: PieceColor::White,
};
pub const BQ: Piece = Piece {
  class: PieceType::Queen,
  color: PieceColor::Black,
};

pub const WK: Piece = Piece {
  class: PieceType::King,
  color: PieceColor::White,
};
pub const BK: Piece = Piece {
  class: PieceType::King,
  color: PieceColor::Black,
};

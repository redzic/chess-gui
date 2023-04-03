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

  // return direction pawns are moving in terms of y-axis
  #[inline]
  pub fn direction(self) -> i32 {
    if self.is_white() {
      -1
    } else {
      1
    }
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

impl Piece {
  pub fn is_pawn(self) -> bool {
    self.class == PieceType::Pawn
  }
  pub fn is_knight(self) -> bool {
    self.class == PieceType::Knight
  }
  pub fn is_bishop(self) -> bool {
    self.class == PieceType::Bishop
  }
  pub fn is_rook(self) -> bool {
    self.class == PieceType::Rook
  }
  pub fn is_queen(self) -> bool {
    self.class == PieceType::Queen
  }
  pub fn is_king(self) -> bool {
    self.class == PieceType::King
  }
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

#[rustfmt::skip]
pub static PAWN_TABLE: [i32; 64] = [
 0,  0,  0,  0,  0,  0,  0,  0,
50, 50, 50, 50, 50, 50, 50, 50,
10, 10, 20, 30, 30, 20, 10, 10,
 5,  5, 10, 25, 25, 10,  5,  5,
 0,  0,  0, 20, 20,  0,  0,  0,
 5, -5,-10,  0,  0,-10, -5,  5,
 5, 10, 10,-20,-20, 10, 10,  5,
 0,  0,  0,  0,  0,  0,  0,  0
];

#[rustfmt::skip]
pub static KNIGHT_TABLE: [i32; 64] = [
 -50,-40,-30,-30,-30,-30,-40,-50,
 -40,-20,  0,  0,  0,  0,-20,-40,
 -30,  0, 10, 15, 15, 10,  0,-30,
 -30,  5, 15, 20, 20, 15,  5,-30,
 -30,  0, 15, 20, 20, 15,  0,-30,
 -30,  5, 10, 15, 15, 10,  5,-30,
 -40,-20,  0,  5,  5,  0,-20,-40,
 -50,-40,-30,-30,-30,-30,-40,-50
];

#[rustfmt::skip]
pub static BISHOP_TABLE: [i32; 64] = [
-20,-10,-10,-10,-10,-10,-10,-20,
-10,  0,  0,  0,  0,  0,  0,-10,
-10,  0,  5, 10, 10,  5,  0,-10,
-10,  5,  5, 10, 10,  5,  5,-10,
-10,  0, 10, 10, 10, 10,  0,-10,
-10, 10, 10, 10, 10, 10, 10,-10,
-10,  5,  0,  0,  0,  0,  5,-10,
-20,-10,-10,-10,-10,-10,-10,-20
];

#[rustfmt::skip]
pub static ROOK_TABLE: [i32; 64] = [
  0,  0,  0,  0,  0,  0,  0,  0,
  5, 10, 10, 10, 10, 10, 10,  5,
 -5,  0,  0,  0,  0,  0,  0, -5,
 -5,  0,  0,  0,  0,  0,  0, -5,
 -5,  0,  0,  0,  0,  0,  0, -5,
 -5,  0,  0,  0,  0,  0,  0, -5,
 -5,  0,  0,  0,  0,  0,  0, -5,
  0,  0,  0,  5,  5,  0,  0,  0
];

#[rustfmt::skip]
pub static QUEEN_TABLE: [i32; 64] = [
-20,-10,-10, -5, -5,-10,-10,-20,
-10,  0,  0,  0,  0,  0,  0,-10,
-10,  0,  5,  5,  5,  5,  0,-10,
 -5,  0,  5,  5,  5,  5,  0, -5,
  0,  0,  5,  5,  5,  5,  0, -5,
-10,  5,  5,  5,  5,  5,  0,-10,
-10,  0,  5,  0,  0,  0,  0,-10,
-20,-10,-10, -5, -5,-10,-10,-20
];

#[rustfmt::skip]
pub static KING_TABLE: [i32; 64] = [
-30,-40,-40,-50,-50,-40,-40,-30,
-30,-40,-40,-50,-50,-40,-40,-30,
-30,-40,-40,-50,-50,-40,-40,-30,
-30,-40,-40,-50,-50,-40,-40,-30,
-20,-30,-30,-40,-40,-30,-30,-20,
-10,-20,-20,-20,-20,-20,-20,-10,
 20, 20,  0,  0,  0,  0, 20, 20,
 20, 30, 10,  0,  0, 10, 30, 20
];

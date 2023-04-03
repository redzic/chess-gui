use std::mem::{self, swap};
use std::ops::{Index, IndexMut};

use sfml::graphics::{
  Color, RectangleShape, RenderTarget, RenderWindow, Shape, Sprite, Texture, Transformable,
};
use sfml::system::{Vector2, Vector2f};
use sfml::window::mouse::Button;
use sfml::window::{ContextSettings, Event, Key, Style, VideoMode};
use sfml::SfBox;

const SQUARE_SIZE: u32 = 100;
const WINDOW_SIZE: u32 = 8 * SQUARE_SIZE;

const DARK: Color = Color::rgb(15, 122, 56);
const LIGHT: Color = Color::rgb(137, 224, 143);
const DARKER: Color = Color::rgb(28, 79, 57);

fn color_mult(color: Color, multiplier: f64) -> Color {
  let f = |x: u8| ((x as f64 * multiplier) as u32).clamp(0, 255) as u8;
  Color::rgb(f(color.r), f(color.g), f(color.b))
}

mod piece;
mod search;

use crate::search::minimax;

use crate::piece::PieceType::*;
use crate::piece::*;

impl Piece {
  // draw the piece on the board
  fn draw(self, (x, y): (u32, u32), window: &mut RenderWindow, texture_map: &[SfBox<Texture>; 12]) {
    let idx = self.color as usize * 6 + self.class as usize;
    let texture = &texture_map[idx];

    // maybe reuse sprites? idk if that affects anything...
    let mut sprite = Sprite::new();
    sprite.set_texture(texture, false);

    let offset = match self.class {
      PieceType::Pawn => PAWN_XOFF,
      PieceType::Knight => KNIGHT_XOFF,
      PieceType::Bishop => BISHOP_XOFF,
      PieceType::Rook => ROOK_XOFF,
      PieceType::Queen => QUEEN_XOFF,
      PieceType::King => KING_XOFF,
    };

    sprite.set_position(Vector2f::new(
      ((x * SQUARE_SIZE) as i32 + offset) as f32,
      (y * SQUARE_SIZE) as f32,
    ));
    window.draw(&sprite);
  }

  fn draw_precise(
    self,
    (x, y): (i32, i32),
    window: &mut RenderWindow,
    texture_map: &[SfBox<Texture>; 12],
  ) {
    let idx = self.color as usize * 6 + self.class as usize;
    let texture = &texture_map[idx];

    // maybe reuse sprites? idk if that affects anything...
    let mut sprite = Sprite::new();
    sprite.set_texture(texture, false);

    let offset = match self.class {
      PieceType::Pawn => PAWN_XOFF,
      PieceType::Knight => KNIGHT_XOFF,
      PieceType::Bishop => BISHOP_XOFF,
      PieceType::Rook => ROOK_XOFF,
      PieceType::Queen => QUEEN_XOFF,
      PieceType::King => KING_XOFF,
    };

    sprite.set_position(Vector2f::new((x + offset) as f32, y as f32));
    window.draw(&sprite);
  }
}

// support Forsyth–Edwards Notation (FEN) notation

type BoardState = Option<Piece>;

#[derive(Copy, Clone, Debug)]
pub struct Board {
  board: [BoardState; 64],
  // outer-
  // [0] - white,
  // [1] - black

  // inner [bool;2] ->
  //  [0] - left
  //  [1] - right (in terms of x-axis, not necessarily from player's perspective)
  castling_rights: [[bool; 2]; 2],

  // en passant
  // square that just moved
  // after apply_move, this should be set to the square of the pawn
  // that is currently there.

  // square that CAN BE CAPTURED (is technically empty)

  // should be set to none after every turn
  en_passant_square: Option<(u32, u32)>,
}

// TODO represent this struct more compactly
#[derive(Copy, Clone, Debug)]
pub struct Move {
  // (x, y)
  from: (u32, u32),
  // (x, y)
  to: (u32, u32),

  // pawn promotion
  promotion: Option<Piece>,
}

impl Move {
  #[inline]
  fn coords(self) -> ((u32, u32), (u32, u32)) {
    (self.from, self.to)
  }

  fn from_coords((x1, y1): (u32, u32), (x2, y2): (u32, u32)) -> Self {
    Self {
      from: (x1, y1),
      to: (x2, y2),
      promotion: None,
    }
  }
}

use PieceColor::*;

impl Board {
  // standard board setup
  fn new() -> Self {
    let mut board = [None; 64];

    for j in 0..8 {
      board[8 + j] = Some(BP);
      board[8 * 6 + j] = Some(WP);
    }

    board[8 * 7] = Some(WR);
    board[8 * 7 + 7] = Some(WR);
    board[0] = Some(BR);
    board[7] = Some(BR);
    board[8 * 7 + 1] = Some(WN);
    board[8 * 7 + 6] = Some(WN);
    board[1] = Some(BN);
    board[6] = Some(BN);
    board[8 * 7 + 2] = Some(WB);
    board[8 * 7 + 5] = Some(WB);
    board[2] = Some(BB);
    board[5] = Some(BB);
    board[3] = Some(BQ);
    board[8 * 7 + 3] = Some(WQ);
    board[4] = Some(BK);
    board[8 * 7 + 4] = Some(WK);

    Self {
      board,
      castling_rights: [[true; 2]; 2],
      en_passant_square: None,
    }
  }

  // debug case 1
  fn new2() -> Self {
    Board {
      board: [
        Some(Piece {
          class: Rook,
          color: Black,
        }),
        Some(Piece {
          class: Knight,
          color: Black,
        }),
        Some(Piece {
          class: Bishop,
          color: Black,
        }),
        Some(Piece {
          class: Queen,
          color: Black,
        }),
        Some(Piece {
          class: King,
          color: Black,
        }),
        Some(Piece {
          class: Bishop,
          color: Black,
        }),
        Some(Piece {
          class: Knight,
          color: Black,
        }),
        Some(Piece {
          class: Rook,
          color: Black,
        }),
        Some(Piece {
          class: Pawn,
          color: Black,
        }),
        Some(Piece {
          class: Pawn,
          color: Black,
        }),
        Some(Piece {
          class: Pawn,
          color: Black,
        }),
        None,
        Some(Piece {
          class: Pawn,
          color: Black,
        }),
        Some(Piece {
          class: Pawn,
          color: Black,
        }),
        Some(Piece {
          class: Pawn,
          color: Black,
        }),
        Some(Piece {
          class: Pawn,
          color: Black,
        }),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(Piece {
          class: Pawn,
          color: Black,
        }),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(Piece {
          class: Pawn,
          color: White,
        }),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(Piece {
          class: Pawn,
          color: White,
        }),
        Some(Piece {
          class: Pawn,
          color: White,
        }),
        Some(Piece {
          class: Pawn,
          color: White,
        }),
        Some(Piece {
          class: Pawn,
          color: White,
        }),
        None,
        Some(Piece {
          class: Pawn,
          color: White,
        }),
        Some(Piece {
          class: Pawn,
          color: White,
        }),
        Some(Piece {
          class: Pawn,
          color: White,
        }),
        Some(Piece {
          class: Rook,
          color: White,
        }),
        Some(Piece {
          class: Knight,
          color: White,
        }),
        Some(Piece {
          class: Bishop,
          color: White,
        }),
        Some(Piece {
          class: Queen,
          color: White,
        }),
        Some(Piece {
          class: King,
          color: White,
        }),
        Some(Piece {
          class: Bishop,
          color: White,
        }),
        Some(Piece {
          class: Knight,
          color: White,
        }),
        Some(Piece {
          class: Rook,
          color: White,
        }),
      ],
      castling_rights: [[true, true], [true, true]],
      en_passant_square: None,
    }
  }

  /// Get copy of board after applying a move.
  fn apply_move(&self, mv: Move) -> Board {
    let ((x1, y1), (x2, y2)) = mv.coords();

    let mut board = *self;

    let en_passant_square = mem::take(&mut board.en_passant_square);

    // handle castling
    if board[(x1, y1)]
      .map(|p| p.class == PieceType::King)
      .unwrap_or(false)
      && (x1 as i32 - x2 as i32).abs() == 2
    {
      debug_assert!(mv.promotion.is_none());

      // direction
      let is_rook_right = x2 > x1;
      let color = board[(x1, y1)].unwrap().color;
      let rank_idx = if color.is_white() { 7 } else { 0 };
      let rook_idx = if is_rook_right { 7 } else { 0 };

      assert!(y1 == y2 && y1 == rank_idx);
      assert!(board.castling_rights[color as usize][is_rook_right as usize]);

      // also cannot castle THROUGH check.

      // assert rook is there
      assert!(board[(rook_idx, y1)]
        .map(|p| p.class == PieceType::Rook)
        .unwrap_or(false));

      // also ensure there are no pieces in between.

      // move king
      board
        .board
        .swap((8 * y1 + x1) as usize, (8 * y2 + x2) as usize);

      let new_rook_x = if is_rook_right { x1 + 1 } else { x1 - 1 };
      board
        .board
        .swap((8 * y1 + rook_idx) as usize, (8 * y1 + new_rook_x) as usize);

      board.castling_rights[color as usize] = [false; 2];

      board
    } else if let Some(promo) = mv.promotion {
      assert!(board[(x1, y1)]
        .map(|p| p.class == PieceType::Pawn)
        .unwrap_or(false));

      // TODO add more checks here

      // let pawn_rank =

      board[(x1, y1)] = None;
      board[(x2, y2)] = Some(promo);

      board
    } else {
      // handle regular move

      // TODO make sure after pawn promotion, you cannot promote to rook
      // and castle with that rook
      // although that shouldn't be possible if the original rooks don't move
      // or get captured, in which case you lose castling rights anyway.

      debug_assert!(mv.promotion.is_none());

      // TODO: && castling rights exist
      // just short-circuit optimization, won't change results
      if let Some(piece) = board[(x1, y1)] {
        let color = piece.color;
        // TODO optimize this
        match piece.class {
          // handle castling
          PieceType::King => {
            board.castling_rights[piece.color as usize] = [false; 2];
          }
          PieceType::Rook => {
            let y_idx = if piece.color.is_black() { 0 } else { 7 };
            if (x1, y1) == (0, y_idx) {
              board.castling_rights[piece.color as usize][0] = false;
            } else if (x1, y1) == (7, y_idx) {
              board.castling_rights[piece.color as usize][1] = false;
            }
          }
          // handle 2 pawn move (en passant)
          PieceType::Pawn if (y1 as i32 - y2 as i32).abs() == 2 => {
            debug_assert!([1, 6].contains(&y1));
            debug_assert!(x1 == x2);

            board.en_passant_square = Some((
              x2,
              (y2 as i32 - piece.color.direction()).try_into().unwrap(),
            ));
          }
          _ => {}
        }

        match (en_passant_square, piece.class) {
          (Some((epx, epy)), PieceType::Pawn) if (epx, epy) == (x2, y2) => {
            debug_assert!(board[(epx, epy)].is_none());

            dbg!((x1, y1), (x2, y2));
            dbg!(&board, en_passant_square);

            let pawn_capture =
              mem::take(&mut board[(epx, (epy as i32 + (!color).direction()).try_into().unwrap())]);
            debug_assert_eq!(
              pawn_capture,
              Some(Piece {
                class: PieceType::Pawn,
                color: !color
              })
            );
            board[(x2, y2)] = board[(x1, y1)];
            board[(x1, y1)] = None;
            board
          }
          _ => {
            board[(x2, y2)] = board[(x1, y1)];
            board[(x1, y1)] = None;
            board
          }
        }
      } else {
        unreachable!("starting square should not be empty in apply_move()")
      }
    }
  }

  fn draw(&self, window: &mut RenderWindow, texture_map: &[SfBox<Texture>; 12]) {
    for i in 0..8u32 {
      for j in 0..8u32 {
        if let Some(piece) = self[(i, j)] {
          piece.draw((i, j), window, texture_map);
        }
      }
    }
  }

  // very simple evaluation, just sum up piece values
  // positive = advantage for white, negative = advantage for black
  fn eval(&self, to_move: PieceColor) -> i32 {
    if is_in_checkmate(&self, to_move) {
      return if to_move.is_white() {
        -1_000_000
      } else {
        1_000_000
      };
    }

    self
      .board
      .iter()
      .enumerate()
      .map(|(idx, square)| match square {
        Some(p) => {
          let mult = if p.color.is_white() { 1 } else { -1 };

          let (magnitude, table) = match p.class {
            Pawn => (100, &PAWN_TABLE),
            Knight => (320, &KNIGHT_TABLE),
            Bishop => (330, &BISHOP_TABLE),
            Rook => (500, &ROOK_TABLE),
            Queen => (900, &QUEEN_TABLE),
            King => (20_000, &KING_TABLE),
          };

          let (x, y) = to_coord(idx as u32);
          let y = if p.color.is_white() { y } else { 7 - y };

          mult * (magnitude + table[(y * 8 + x) as usize])
        }
        None => 0,
      })
      .sum()
  }

  fn moves_for_player(&self, color: PieceColor) -> Vec<Move> {
    let mut moves = vec![];

    for x in 0..8u32 {
      for y in 0..8u32 {
        if let Some(piece) = self[(x, y)] {
          if piece.color == color {
            let p_mvs = moves_for_piece(self, (x, y));
            moves.extend_from_slice(&*p_mvs);
          }
        }
      }
    }

    // TODO deduplicate this code

    // retain moves that don't put us in check
    // closure returns false for illegal moves, true for legal
    moves.retain(|&mv| {
      is_move_legal(self, mv) && {
        let board_after_move = self.apply_move(mv);
        !is_in_check(&board_after_move, color)
      }
    });

    moves
  }
}

impl Index<usize> for Board {
  type Output = Option<Piece>;
  fn index(&self, index: usize) -> &Self::Output {
    &self.board[index]
  }
}

impl Index<u32> for Board {
  type Output = Option<Piece>;
  fn index(&self, index: u32) -> &Self::Output {
    &self.board[index as usize]
  }
}

impl<T: Into<u32>> Index<(T, T)> for Board {
  type Output = Option<Piece>;
  fn index(&self, (x, y): (T, T)) -> &Self::Output {
    &self.board[8 * y.into() as usize + x.into() as usize]
  }
}

impl IndexMut<usize> for Board {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.board[index]
  }
}

impl IndexMut<u32> for Board {
  fn index_mut(&mut self, index: u32) -> &mut Self::Output {
    &mut self.board[index as usize]
  }
}

impl<T: Into<u32>> IndexMut<(T, T)> for Board {
  fn index_mut(&mut self, (x, y): (T, T)) -> &mut Self::Output {
    &mut self.board[8 * y.into() as usize + x.into() as usize]
  }
}

#[inline(always)]
fn sort2<T: Copy + Ord>(x: T, y: T) -> (T, T) {
  if x < y {
    (x, y)
  } else {
    (y, x)
  }
}

fn is_bishop_move_legal(board: &Board, (x1, y1): (u32, u32), (x2, y2): (u32, u32)) -> bool {
  if (x1 as i32 - x2 as i32).abs() == (y1 as i32 - y2 as i32).abs() {
    let n_rows = {
      let (yy1, yy2) = sort2(y1 as i32, y2 as i32);
      yy2 - yy1 - 1
    };

    let (mut x1, mut x2, mut y1, mut y2) = (x1, x2, y1, y2);

    if y1 > y2 {
      // swap (x1, y1) and (x2, y2)
      swap(&mut x1, &mut x2);
      swap(&mut y1, &mut y2);
    }

    let stride = if x1 < x2 { 9 } else { 7 };

    let mut idx = 8 * y1 + x1;
    for _ in 0..n_rows {
      idx += stride;
      if board[idx].is_some() {
        return false;
      }
    }

    true
  } else {
    false
  }
}

// doesn't check for self-capture as that is checked universally for all moves.
fn is_rook_move_legal(board: &Board, (x1, y1): (u32, u32), (x2, y2): (u32, u32)) -> bool {
  let x_match = x1 == x2;
  if x_match ^ (y1 == y2) {
    let (x1, x2) = sort2(x1, x2);
    let (y1, y2) = sort2(y1, y2);

    if x_match {
      // [(x1, y1 + 1), (x1, y2 - 1)]
      for y in y1 + 1..=y2 - 1 {
        if board[(x1, y)].is_some() {
          return false;
        }
      }
    } else {
      // [(x1 + 1, y1), (x2 - 1, y1)]
      for x in x1 + 1..=x2 - 1 {
        if board[(x, y1)].is_some() {
          return false;
        }
      }
    }

    true
  } else {
    false
  }
}

#[inline(always)]
pub const fn to_offset(x: i32, y: i32) -> i32 {
  8 * y + x
}

#[inline(always)]
pub const fn to_coord(idx: u32) -> (u32, u32) {
  (idx % 8, idx / 8)
}

// pub fn get_offsets()

// maybe keep track of what moves were played so that it is easy
// to revert them, to avoid making copies of the board to check
// for check.

pub fn is_in_checkmate(board: &Board, player: PieceColor) -> bool {
  // loop through possible all moves, see if any of them do not put you in check

  for i in 0..=63 {
    match board[i as usize] {
      Some(p) if p.color == player => {
        let (x, y) = to_coord(i);
        let moves = moves_for_piece(board, (x, y));
        for mv in moves {
          if !is_in_check(&board.apply_move(mv), player) {
            return false;
          }
        }
      }
      _ => {}
    }
  }

  true
}

pub fn is_in_check(board: &Board, player: PieceColor) -> bool {
  // loop through all opponent pieces, except for king (debug assert maybe?).

  // for each of the opponent's pieces, check if any of the squares they
  // attack cover our king.

  // find index of player's king
  let (kx, ky) = to_coord(
    board
      .board
      .iter()
      .position(|&p| {
        p == Some(Piece {
          class: PieceType::King,
          color: player,
        })
      })
      .expect("king should always exist on board") as u32,
  );

  // could also maybe just keep track of the board state some other way
  // to avoid looping through the board?
  // but I think this is fine for now...

  // loop through opponent's pieces
  for i in 0..=63 {
    match board[i as usize] {
      // Some(p) if p.color != player && p.class != PieceType::King => {
      Some(p) if p.color != player => {
        // check if any of their moves covers our king
        let squares = moves_for_piece(board, to_coord(i));

        for mv in squares {
          if mv.to == (kx, ky) {
            return true;
          }
        }
      }
      _ => {}
    }
  }

  false
}

#[inline(always)]
pub fn inbounds(x: i32, y: i32) -> bool {
  (0..=7).contains(&x) && (0..=7).contains(&y)
}

// mv!(..., ..., (x, y))
macro_rules! mv {
  ($x:expr, $y:expr, $from:expr) => {
    Move::from_coords($from, ($x, $y))
  };
}

fn moves_for_sliding_piece(
  board: &Board,
  (x, y): (u32, u32),
  directions: &[(i32, i32)],
) -> Vec<Move> {
  debug_assert!(board[(x, y)]
    .map(|p| matches!(
      p.class,
      PieceType::Rook | PieceType::Bishop | PieceType::Queen
    ))
    .unwrap_or(false));

  if let Some(p) = board[(x, y)] {
    let mut moves = vec![];

    for (xd, yd) in directions {
      let mut xt = x as i32 + xd;
      let mut yt = y as i32 + yd;
      while inbounds(xt, yt) {
        if let Some(p2) = board[(xt as u32, yt as u32)] {
          if p2.color != p.color {
            moves.push(mv!(xt as u32, yt as u32, (x, y)));
          }
          break;
        } else {
          moves.push(mv!(xt as u32, yt as u32, (x, y)));
        }

        xt += xd;
        yt += yd;
      }
    }

    moves
  } else {
    unreachable!()
  }
}

static ROOK_DIRECTIONS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, 1), (0, -1)];
static BISHOP_DIRECTIONS: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
static QUEEN_DIRECTIONS: [(i32, i32); 8] = [
  (-1, 0),
  (1, 0),
  (0, 1),
  (0, -1),
  (-1, -1),
  (-1, 1),
  (1, -1),
  (1, 1),
];

// return vector of final coordinates for piece
// TODO (important) update this to handle pawn promotion as a possible move
// otherwise checking for checkmate will be wrong in some cases.
// fn moves_for_piece(board: &Board, (x, y): (u32, u32)) -> Vec<(u32, u32)> {
fn moves_for_piece(board: &Board, (x, y): (u32, u32)) -> Vec<Move> {
  // we generate offsets, then maybe also check further legality of the move?
  // i.e. we do not put our own king in check by making this move
  if let Some(p) = board[(x, y)] {
    match p.class {
      PieceType::Knight => {
        let mut moves = vec![];
        let possible_moves = [
          (-2, 1),
          (-2, -1),
          (2, 1),
          (2, -1),
          (1, -2),
          (-1, -2),
          (1, 2),
          (-1, 2),
        ];

        // filter out moves that are off the board and self-capture
        for (xoff, yoff) in possible_moves {
          let xn = x as i32 + xoff;
          let yn = y as i32 + yoff;
          if inbounds(xn, yn)
            && board[(xn as u32, yn as u32)]
              .map(|p2| p2.color != p.color)
              .unwrap_or(true)
          {
            moves.push(mv!(xn as u32, yn as u32, (x, y)));
          }
        }

        moves
      }
      PieceType::Rook => moves_for_sliding_piece(board, (x, y), &ROOK_DIRECTIONS),
      PieceType::Bishop => moves_for_sliding_piece(board, (x, y), &BISHOP_DIRECTIONS),
      PieceType::Queen => moves_for_sliding_piece(board, (x, y), &QUEEN_DIRECTIONS),
      PieceType::King => {
        let mut moves = vec![];

        let move_offsets = [
          (-1, -1),
          (0, -1),
          (1, -1),
          (-1, 0),
          (1, 0),
          (-1, 1),
          (0, 1),
          (1, 1),
        ];

        for (xoff, yoff) in move_offsets {
          let (xd, yd) = (x as i32 + xoff, y as i32 + yoff);
          if inbounds(xd, yd)
            && board[(xd as u32, yd as u32)]
              .map(|p2| p2.color != p.color)
              .unwrap_or(true)
          {
            moves.push(mv!(xd as u32, yd as u32, (x, y)));
          }
        }

        let rank_yidx = if p.color.is_white() { 7u32 } else { 0 };
        // if board.castling_rights[p.color as usize] && y == rank_yidx {
        if y == rank_yidx {
          // add castling moves
          // TODO update castling rights for each rook
          // TODO maybe also check that king is on original square,
          // but maybe this can be done with accurate castling rights.

          for rook_idx in [0u32, 7] {
            if let (
              true,
              Some(Piece {
                class: PieceType::Rook,
                color,
              }),
            ) = (
              board.castling_rights[p.color as usize][(rook_idx != 0) as usize],
              board[(rook_idx, rank_yidx)],
            ) {
              if color == p.color {
                // maybe add

                let mut idxs: [u32; 2] = [x, rook_idx];
                idxs.sort();

                if !do_pieces_exist_x1x2(board, rank_yidx, (idxs[0] + 1, idxs[1] - 1)) {
                  // if no pieces exist in between and we have castling rights, we can
                  // add this as a move
                  let is_rook_right = rook_idx > x;
                  let king_xoff: i32 = if is_rook_right { 2 } else { -2 };

                  moves.push(mv!(
                    (x as i32 + king_xoff).try_into().unwrap(),
                    rank_yidx,
                    (x, y)
                  ));
                }
              }
            }
          }
        }

        moves
      }
      PieceType::Pawn => {
        let mut moves = vec![];

        let direction = if p.color.is_white() { -1 } else { 1 };

        let last_rank = if p.color.is_white() { 0 } else { 7 };

        // basic move, push forward 1
        let (bx, by) = (x as i32, y as i32 + direction);
        if inbounds(bx, by) && board[(bx as u32, by as u32)].is_none() {
          if by == last_rank {
            for pt in PROMO_OPTS {
              moves.push(Move {
                from: (x, y),
                to: (bx as u32, by as u32),
                promotion: Some(Piece {
                  class: pt,
                  color: p.color,
                }),
              });
            }
          } else {
            moves.push(mv!(bx as u32, by as u32, (x, y)));
          }
        }

        // push 2 if on rank 2
        let push2_rank = if p.color.is_white() { 6 } else { 1 };
        if y == push2_rank {
          // sanity check; this should always be in bounds because of the
          // rank the pawn is on.
          debug_assert!(inbounds(x as i32, y as i32 + 2 * direction));

          let (px, py) = (x, (y as i32 + 2 * direction) as u32);
          if board[(px, py)].is_none() && board[(px, (y as i32 + direction) as u32)].is_none() {
            moves.push(mv!(px, py, (x, y)));
          }
        }

        // sideways attacking moves
        for xoff in [-1, 1] {
          let (ax, ay) = (x as i32 + xoff, y as i32 + direction);
          if inbounds(ax, ay)
            && match board[(ax as u32, ay as u32)] {
              Some(p2) => p2.color != p.color,
              None => Some((ax as u32, ay as u32)) == board.en_passant_square,
            }
          {
            // TODO: dedup code from previous?
            if ay == last_rank {
              for pt in PROMO_OPTS {
                moves.push(Move {
                  from: (x, y),
                  to: (ax as u32, ay as u32),
                  promotion: Some(Piece {
                    class: pt,
                    color: p.color,
                  }),
                });
              }
            } else {
              moves.push(mv!(ax as u32, ay as u32, (x, y)));
            }
          }
        }

        moves
      }
    }
  } else {
    unreachable!("function should not be called on empty square")
  }
}

// check if any pieces exist on a certain rank between x1 and x2 (inclusive)
fn do_pieces_exist_x1x2(board: &Board, rank_idx: u32, (x1, x2): (u32, u32)) -> bool {
  debug_assert!(x1 <= x2);

  for x in x1..=x2 {
    if board[(x, rank_idx)].is_some() {
      return true;
    }
  }

  false
}

fn is_move_legal(board: &Board, mv: Move) -> bool {
  let ((x1, y1), (x2, y2)) = mv.coords();

  // TODO do not allow moves that put your king in check
  if (x1, y1) == (x2, y2) {
    return false;
  }

  // TODO maybe check other obviously ridiculous scenarios here as well,
  // possibly as a debug assert?

  // ensure (x1, y1) exists and that if it is not a pawn, it doesn't
  // have a promotion.
  assert!(board[(x1, y1)]
    .map(|p| if !p.is_pawn() {
      mv.promotion.is_none()
    } else {
      true
    })
    .unwrap_or(false));

  if let Some(piece) = board[(x1, y1)] {
    match piece.class {
      PieceType::Pawn => {
        // en pessant as well...

        let y_dist = || y2 as i32 - y1 as i32;

        // rank2 is the rank where 2 moves as a pawn is allowed.
        let (rank2, file_range, direction) = match piece.color {
          PieceColor::White => (6, (-2..=-1), -1),
          PieceColor::Black => (1, (1..=2), 1),
        };

        // TODO clean up this code
        (if let Some(captured_piece) = board[(x2, y2)] {
          // regular capture
          captured_piece.color != piece.color
            && (x1 as i32 - x2 as i32).abs() == 1
            && y_dist() == direction
        } else if x1 != x2 {
          // en passant

          // due to order of if statement, it is guaranteed that board[(x2, y2)] is None
          // in this branch

          board.en_passant_square == Some((x2, y2))
            && (x1 as i32 - x2 as i32).abs() == 1
            && y_dist() == direction
        } else if y1 == rank2 {
          // move from starting square (can move 1 or 2 squares forward)
          x1 == x2
            && file_range.contains(&y_dist())
            && (1..=y_dist())
              .all(|r_off| board[(x1, (y1 as i32 + r_off * direction) as u32)].is_none())
        } else {
          // basic pawn move, 1 forward
          (x2, y2 as i32) == (x1, y1 as i32 + direction)
        }) && ({
          let last_rank = if piece.color.is_white() { 0 } else { 7 };

          if y2 == last_rank {
            // promotion exists and is valid
            mv.promotion
              // TODO: optimize to just simple range check instead of loop
              .map(|pr| PROMO_OPTS.contains(&pr.class) && pr.color == piece.color)
              .unwrap_or(false)
          } else {
            true
          }
        })
      }
      PieceType::Knight => {
        let xdist = (x1 as i32 - x2 as i32).abs() - 1;
        let ydist = (y1 as i32 - y2 as i32).abs() - 1;

        // ensure all bits except for lsb are 0
        // only need to check one of xdist or ydist, since
        // if all bits except lsb are 0, that means
        // it REQUIRES all top bits of ydist to also be 0,
        // otherwise xdist ^ ydist != 1.

        xdist ^ ydist == 1 && xdist & !1 == 0
      }
      PieceType::Bishop => is_bishop_move_legal(board, (x1, y1), (x2, y2)),
      PieceType::Rook => is_rook_move_legal(board, (x1, y1), (x2, y2)),
      PieceType::Queen => {
        let xdist = (x1 as i32 - x2 as i32).abs();
        let ydist = (y1 as i32 - y2 as i32).abs();

        if xdist == ydist {
          is_bishop_move_legal(board, (x1, y1), (x2, y2))
        } else if (xdist == 0) ^ (ydist == 0) {
          is_rook_move_legal(board, (x1, y1), (x2, y2))
        } else {
          false
        }
      }
      PieceType::King => {
        let xdist = (x1 as i32 - x2 as i32).abs();

        match xdist {
          0 | 1 => {
            let ydist = (y1 as i32 - y2 as i32).abs();
            ydist <= 1
          }
          2 => {
            let rank_idx = if piece.color.is_white() { 7 } else { 0 };
            let file_idx = 4;
            if y1 != y2 || y1 != rank_idx {
              return false;
            }
            if x1 != file_idx {
              return false;
            }

            let is_rook_right = x2 > x1;

            // ensure castling rights exist
            if !board.castling_rights[piece.color as usize][is_rook_right as usize] {
              return false;
            }

            // in terms of x index, not necessarily from player's perspective
            let rook_idx = if is_rook_right { 7 } else { 0 };

            if let Some(Piece {
              class: PieceType::Rook,
              color,
            }) = board[(rook_idx, rank_idx)]
            {
              if color != piece.color {
                return false;
              }
            } else {
              return false;
            }

            let mut xidx = [x1, rook_idx];
            xidx.sort();

            // TODO also check castle THROUGH check here
            if do_pieces_exist_x1x2(board, y1, (xidx[0] + 1, xidx[1] - 1)) {
              return false;
            }

            true
          }
          _ => false,
        }
      }
    }
  } else {
    // shouldn't happen
    debug_assert!(false);

    false
  }
}

static PROMO_OPTS: [PieceType; 4] = [Knight, Bishop, Rook, Queen];

fn draw_board(
  board: Board,
  window: &mut RenderWindow,
  texture_map: &[SfBox<Texture>; 12],
  selection: &Option<((u32, u32), (i32, i32), Vec<Move>)>,
  draw_selection: bool,
) {
  window.clear(LIGHT);

  let mut rect = RectangleShape::new();
  rect.set_fill_color(DARK);
  rect.set_size(Vector2::new(SQUARE_SIZE as f32, SQUARE_SIZE as f32));

  // draw dark squares
  for i in 0..8 {
    for j in 0..8 {
      if (i ^ j) & 1 != 0 {
        rect.set_position(Vector2f::new(
          (SQUARE_SIZE * i) as f32,
          (SQUARE_SIZE * j) as f32,
        ));
        window.draw(&rect);
      }
    }
  }

  match (selection, draw_selection) {
    (Some(((sx, sy), (xd, yd), moves)), true) => {
      // no if-let chain on stable :(
      for mv in moves {
        let (xd, yd) = mv.to;

        let is_dark = (xd ^ yd) & 1 != 0;
        // TODO maybe blend with another color slightly or something?
        let color = color_mult(if is_dark { DARK } else { LIGHT }, 0.4);
        rect.set_fill_color(color);

        rect.set_position(Vector2f::new(
          (SQUARE_SIZE * xd) as f32,
          (SQUARE_SIZE * yd) as f32,
        ));
        window.draw(&rect);
      }

      debug_assert!(board[(*sx, *sy)].is_some());

      let mut board_copy = board;

      if let Some(selected_piece) = mem::take(&mut board_copy[(*sx, *sy)]) {
        board_copy.draw(window, texture_map);
        selected_piece.draw_precise(
          (
            *xd - (SQUARE_SIZE as i32 / 2),
            *yd - (SQUARE_SIZE as i32 / 2),
          ),
          window,
          texture_map,
        );
      } else {
        board_copy.draw(window, texture_map);
      }
    }
    _ => {
      board.draw(window, texture_map);
    }
  }
}

fn main() {
  let max_aa = sfml::graphics::RenderTexture::maximum_antialiasing_level();

  let mut window = RenderWindow::new(
    (8 * SQUARE_SIZE, 8 * SQUARE_SIZE),
    "Chess AI",
    Style::CLOSE,
    &ContextSettings {
      antialiasing_level: max_aa,
      ..Default::default()
    },
  );

  let vm = VideoMode::desktop_mode();

  // seems to run smoother with vsync off
  window.set_vertical_sync_enabled(false);
  window.set_position(Vector2::new(
    ((vm.width - WINDOW_SIZE) / 2) as i32,
    ((vm.height - WINDOW_SIZE) / 2) as i32,
  ));

  let mut texture = Texture::from_file("./resources/b_king.png").unwrap();
  texture.set_smooth(true);

  let texture_map = [
    Texture::from_file("./resources/w_pawn.png").unwrap(),
    Texture::from_file("./resources/w_knight.png").unwrap(),
    Texture::from_file("./resources/w_bishop.png").unwrap(),
    Texture::from_file("./resources/w_rook.png").unwrap(),
    Texture::from_file("./resources/w_queen.png").unwrap(),
    Texture::from_file("./resources/w_king.png").unwrap(),
    // -- black pieces
    Texture::from_file("./resources/b_pawn.png").unwrap(),
    Texture::from_file("./resources/b_knight.png").unwrap(),
    Texture::from_file("./resources/b_bishop.png").unwrap(),
    Texture::from_file("./resources/b_rook.png").unwrap(),
    Texture::from_file("./resources/b_queen.png").unwrap(),
    Texture::from_file("./resources/b_king.png").unwrap(),
  ];

  let mut board = Board::new();
  let mut selection: Option<((u32, u32), (i32, i32), Vec<Move>)> = None;

  let mut to_move = PieceColor::White;

  let mut depth_white = 1;
  let mut depth_black = 1;

  loop {
    while let Some(event) = window.poll_event() {
      match event {
        Event::Closed
        | Event::KeyPressed {
          code: Key::Escape, ..
        } => return,

        // Event::KeyPressed { code: Key::, .. }=>{}

        // dump board
        Event::KeyPressed {
          code: Key::Space, ..
        } => {
          println!("Current eval: {}", board.eval(to_move));

          // shit does not work properly in regards to check,
          // search does not seem to consider legal moves.

          let depth = if to_move.is_white() {
            depth_white
          } else {
            depth_black
          };

          let search_result = minimax(board, depth - 1, to_move);
          println!("Minimax (depth: {} ply): {:?}", depth, search_result);

          board = board.apply_move(search_result.0);

          to_move = !to_move;

          // TODO: deduplicate code

          // gg
          if is_in_checkmate(&board, to_move) {
            println!("Checkmate! {:?} wins.", !to_move);
            return;
          }
        }

        Event::KeyPressed { code, shift, .. } => {
          if (Key::Num0 as u32..=Key::Num9 as u32).contains(&(code as u32)) {
            let num = code as u32 - Key::Num0 as u32;

            if num != 0 {
              if shift {
                depth_black = num;
                println!("[Info] Search depth (Black) set to {} ply", num)
              } else {
                depth_white = num;
                println!("[Info] Search depth (White) set to {} ply", num)
              }
            }
          }
        }

        Event::MouseMoved { x, y } => {
          if let Some((_, point, _)) = &mut selection {
            *point = (x, y);
          }
        }

        Event::MouseButtonPressed {
          button: Button::Left,
          x,
          y,
        } => {
          let (xn, yn) = (x as u32 / SQUARE_SIZE, y as u32 / SQUARE_SIZE);

          // don't allow selecting empty squares
          if let Some(piece) = board[(xn, yn)] {
            // only allow selecting color to move
            if piece.color == to_move {
              // ok something is not working...
              let mut moves = moves_for_piece(&board, (xn, yn));

              // retain moves that don't put us in check
              // closure returns false for illegal moves, true for legal
              moves.retain(|&mv| {
                let board_after_move = board.apply_move(mv);
                !is_in_check(&board_after_move, to_move)
              });

              selection = Some(((xn, yn), (x, y), moves));
            }
          }
        }

        Event::MouseButtonReleased {
          button: Button::Left,
          x,
          y,
        } => {
          let (xn, yn) = (x as u32 / SQUARE_SIZE, y as u32 / SQUARE_SIZE);

          if let Some(((ox, oy), _, _)) = selection {
            // TODO if-let or something, this is too much nesting
            if let Some(old_piece) = board[(ox, oy)] {
              let old_color = old_piece.color;

              let new_piece_isnt_same_color = || {
                board[(xn, yn)]
                  .map(|p| p.color != old_color)
                  .unwrap_or(true)
              };

              if (ox, oy) != (xn, yn) && new_piece_isnt_same_color() {
                // move needs to be legal AND cannot put us in check after we do it

                // check for pawn promotion, if so, don't apply move, change state to
                // menu selection and then apply move after selecting pawn move.

                let pawn_last_rank = if old_color.is_white() { 0 } else { 7 };

                let is_pawn_promotion = || {
                  old_piece.is_pawn() && yn == pawn_last_rank && (oy as i32 - yn as i32).abs() == 1
                };

                let mut promotion: Option<Piece> = None;

                if is_pawn_promotion() {
                  {
                    let mut board_copy = board;
                    // move pawn for display purposes
                    let pawn = mem::take(&mut board_copy[(ox, oy)]);
                    board_copy[(xn, yn)] = pawn;

                    draw_board(board_copy, &mut window, &texture_map, &selection, false);
                  }

                  // TODO dedup these 2 lines (rect setup)
                  let mut rect = RectangleShape::new();
                  rect.set_size(Vector2::new(SQUARE_SIZE as f32, SQUARE_SIZE as f32));

                  // render code goes here now
                  rect.set_fill_color(DARKER);
                  for i in 0..4i32 {
                    let draw_x = SQUARE_SIZE as i32 * 4 + (i - 2) * SQUARE_SIZE as i32;
                    let draw_y = (SQUARE_SIZE * 4 - SQUARE_SIZE / 2) as i32;

                    rect.set_position(Vector2f::new(draw_x as f32, draw_y as f32));

                    window.draw(&rect);
                    Piece {
                      class: PROMO_OPTS[i as usize],
                      color: to_move,
                    }
                    .draw_precise((draw_x, draw_y), &mut window, &texture_map);
                  }

                  window.display();

                  'pawn: loop {
                    while let Some(event) = window.poll_event() {
                      match event {
                        // TODO dedup with main loop?
                        Event::Closed
                        | Event::KeyPressed {
                          code: Key::Escape, ..
                        } => return,

                        Event::MouseButtonPressed {
                          button: Button::Left,
                          x,
                          y,
                        } => {
                          // Refer to drawing pawn selection code to find where these
                          // values come from

                          let min_x = 2 * SQUARE_SIZE;
                          let max_x = 6 * SQUARE_SIZE;

                          let min_y = 4 * SQUARE_SIZE - SQUARE_SIZE / 2;
                          let max_y = min_y + SQUARE_SIZE;

                          assert!(selection.is_some());

                          if (min_x..=max_x).contains(&(x as u32))
                            && (min_y..=max_y).contains(&(y as u32))
                          {
                            let normalized_x = x as u32 - min_x;
                            let square_idx = normalized_x / SQUARE_SIZE;

                            let selected_promo = PROMO_OPTS[square_idx as usize];

                            promotion = Some(Piece {
                              class: selected_promo,
                              color: to_move,
                            });
                            break 'pawn;
                          }
                        }

                        _ => {}
                      }
                    }
                  }
                }

                let mv = dbg!(Move {
                  from: (ox, oy),
                  to: (xn, yn),
                  promotion,
                });

                let board_after_move = || board.apply_move(mv);

                if dbg!(is_move_legal(&board, mv))
                  && !dbg!(is_in_check(&board_after_move(), to_move))
                {
                  board = board_after_move();

                  println!("({ox}, {oy}) -> ({x}, {y})");

                  to_move = !to_move;

                  // gg
                  if is_in_checkmate(&board, to_move) {
                    println!("Checkmate! {:?} wins.", !to_move);
                    return;
                  }

                  println!("{:?}", to_move);
                  // println!("{to_move:?} in check? {}", is_in_check(&board, to_move));
                } else {
                  println!("Illegal move!");
                }
              }
            }

            selection = None;

            dbg!(board.en_passant_square);
          }
        }
        _ => {}
      }
    }

    draw_board(board, &mut window, &texture_map, &selection, true);

    window.display()

    // -en passant
    // -castle through check
    // -50 move rule
    // -stalemate
  }
}

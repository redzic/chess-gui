use std::mem::swap;
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

mod piece;

use crate::piece::*;

impl Piece {
  // draw the piece on the board
  fn draw(self, (x, y): (u32, u32), window: &mut RenderWindow, texture_map: &[SfBox<Texture>; 12]) {
    let idx = self.color as usize * 6 + self.class as usize;
    let texture = &texture_map[idx];

    // maybe reuse sprites? idk if that affects anything...
    let mut sprite = Sprite::new();
    sprite.set_texture(&texture, false);

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
}

// support Forsyth–Edwards Notation (FEN) notation

type BoardState = Option<Piece>;

#[derive(Copy, Clone)]
pub struct Board {
  board: [BoardState; 64],
}

impl Board {
  fn new() -> Self {
    let mut board = [None; 64];

    for j in 0..8 {
      board[8 * 1 + j] = Some(BP);
      board[8 * 6 + j] = Some(WP);
    }

    board[8 * 7 + 0] = Some(WR);
    board[8 * 7 + 7] = Some(WR);
    board[8 * 0 + 0] = Some(BR);
    board[8 * 0 + 7] = Some(BR);
    board[8 * 7 + 1] = Some(WN);
    board[8 * 7 + 6] = Some(WN);
    board[8 * 0 + 1] = Some(BN);
    board[8 * 0 + 6] = Some(BN);
    board[8 * 7 + 2] = Some(WB);
    board[8 * 7 + 5] = Some(WB);
    board[8 * 0 + 2] = Some(BB);
    board[8 * 0 + 5] = Some(BB);
    board[8 * 0 + 3] = Some(BQ);
    board[8 * 7 + 3] = Some(WQ);
    board[8 * 0 + 4] = Some(BK);
    board[8 * 7 + 4] = Some(WK);

    Self { board }
  }

  /// Get copy of board after applying a move.
  fn apply_move(&self, (x1, y1): (u32, u32), (x2, y2): (u32, u32)) -> Board {
    let mut board = *self;
    board[(x2, y2)] = board[(x1, y1)];
    board[(x1, y1)] = None;
    board
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
          if !is_in_check(&board.apply_move((x, y), mv), player) {
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

        for (sx, sy) in squares {
          if (sx, sy) == (kx, ky) {
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

fn moves_for_sliding_piece(
  board: &Board,
  (x, y): (u32, u32),
  directions: &[(i32, i32)],
) -> Vec<(u32, u32)> {
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
            moves.push((xt as u32, yt as u32));
          }
          break;
        } else {
          moves.push((xt as u32, yt as u32));
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

// vector of offsets maybe?
fn moves_for_piece(board: &Board, (x, y): (u32, u32)) -> Vec<(u32, u32)> {
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
            moves.push((xn as u32, yn as u32));
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
            moves.push((xd as u32, yd as u32));
          }
        }

        moves
      }
      PieceType::Pawn => {
        let mut moves = vec![];

        let direction = if p.color == PieceColor::White { -1 } else { 1 };

        // basic move, push forward 1
        let (bx, by) = (x as i32, y as i32 + direction);
        if inbounds(bx, by) && board[(bx as u32, by as u32)].is_none() {
          moves.push((bx as u32, by as u32));
        }

        // push 2 if on rank 2
        let push2_rank = if p.color == PieceColor::White { 6 } else { 1 };
        if y == push2_rank {
          // sanity check; this should always be in bounds because of the
          // rank the pawn is on.
          debug_assert!(inbounds(x as i32, y as i32 + 2 * direction));

          let (px, py) = (x, (y as i32 + 2 * direction) as u32);
          if board[(px, py)].is_none() && board[(px, (y as i32 + direction) as u32)].is_none() {
            moves.push((px, py));
          }
        }

        // sideways attacking moves
        for xoff in [-1, 1] {
          let (ax, ay) = (x as i32 + xoff, y as i32 + direction);
          if inbounds(ax, ay)
            && board[(ax as u32, ay as u32)]
              .map(|p2| p2.color != p.color)
              .unwrap_or(false)
          {
            moves.push((ax as u32, ay as u32));
          }
        }

        moves
      }
    }
  } else {
    unreachable!("function should not be called on empty square")
  }
}

fn is_move_legal(board: &Board, (x1, y1): (u32, u32), (x2, y2): (u32, u32)) -> bool {
  // TODO do not allow moves that put your king in check
  if (x1, y1) == (x2, y2) {
    return false;
  }

  // TODO maybe check other obviously ridiculous scenarios here as well,
  // possibly as a debug assert?

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

        if let Some(captured_piece) = board[(x2, y2)] {
          captured_piece.color != piece.color
            && (x1 as i32 - x2 as i32).abs() == 1
            && y_dist() == direction
        } else if y1 == rank2 {
          x1 == x2
            && file_range.contains(&y_dist())
            && (1..=y_dist())
              .all(|r_off| board[(x1, (y1 as i32 + r_off * direction) as u32)].is_none())
        } else {
          (x2, y2 as i32) == (x1, y1 as i32 + direction)
        }
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
        let ydist = (y1 as i32 - y2 as i32).abs();

        xdist <= 1 && ydist <= 1
      }
    }
  } else {
    // shouldn't happen
    debug_assert!(false);

    false
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

  window.set_vertical_sync_enabled(true);
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
  let mut selection: Option<((u32, u32), Vec<(u32, u32)>)> = None;
  let mut to_move = PieceColor::White;

  loop {
    while let Some(event) = window.poll_event() {
      match event {
        Event::Closed
        | Event::KeyPressed {
          code: Key::Escape, ..
        } => return,
        Event::MouseButtonPressed {
          button: Button::Left,
          x,
          y,
        } => {
          let (x, y) = (x as u32 / SQUARE_SIZE, y as u32 / SQUARE_SIZE);

          // println!("click: ({}, {})", x, y);

          if let Some(((ox, oy), _)) = selection {
            if let Some(old_piece) = board[(ox, oy)] {
              let old_color = old_piece.color;

              let new_piece_isnt_same_color =
                || board[(x, y)].map(|p| p.color != old_color).unwrap_or(true);

              if (ox, oy) != (x, y) && new_piece_isnt_same_color() {
                // move needs to be legal AND cannot put us in check after we do it

                let board_after_move = || board.apply_move((ox, oy), (x, y));

                if is_move_legal(&board, (ox, oy), (x, y))
                  && !is_in_check(&board_after_move(), to_move)
                {
                  // TODO maybe abstract this away?
                  // move piece
                  board[(x, y)] = board[(ox, oy)];
                  board[(ox, oy)] = None;

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
          } else {
            // don't allow selecting empty squares
            if let Some(piece) = board[(x, y)] {
              // only allow selecting color to move
              if piece.color == to_move {
                // ok something is not working...
                let mut moves = moves_for_piece(&board, (x, y));

                // retain moves that don't put us in check
                // closure returns false for illegal moves, true for legal
                moves.retain(|&(x2, y2)| {
                  let board_after_move = board.apply_move((x, y), (x2, y2));
                  !is_in_check(&board_after_move, to_move)
                });

                selection = Some(((x, y), moves));
              }
            }
          }
        }
        _ => {}
      }
    }

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

    // (draw other squares here)
    if let Some(((_, _), offset_array)) = &selection {
      rect.set_fill_color(Color::rgb(255, 0, 0));
      for (xd, yd) in offset_array {
        // let (xd, yd) = to_coord(((*y as i32) * 8 + (*x as i32) + offset) as u32);
        rect.set_position(Vector2f::new(
          (SQUARE_SIZE * *xd) as f32,
          (SQUARE_SIZE * *yd) as f32,
        ));
        window.draw(&rect);
      }
    }

    board.draw(&mut window, &texture_map);

    window.display()
  }
}

use std::mem::swap;
use std::ops::{Index, IndexMut, Not};

use sfml::graphics::{
  Color, RectangleShape, RenderTarget, RenderWindow, Shape, Sprite, Texture, Transformable,
};
use sfml::system::{Vector2, Vector2f};
use sfml::window::mouse::Button;
use sfml::window::{ContextSettings, Event, Key, Style, VideoMode};
use sfml::SfBox;

const SQUARE_SIZE: u32 = 100;
const WINDOW_SIZE: u32 = 8 * SQUARE_SIZE;

const DARK: Color = Color::rgb(30, 31, 79);
const LIGHT: Color = Color::rgb(132, 134, 232);

// x-offsets for displaying
const PAWN_XOFF: i32 = 10;
const KNIGHT_XOFF: i32 = 5;
const ROOK_XOFF: i32 = 6;
const BISHOP_XOFF: i32 = 3;
const QUEEN_XOFF: i32 = -2;
const KING_XOFF: i32 = 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PieceColor {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PieceType {
  Pawn = 0,
  Knight = 1,
  Bishop = 2,
  Rook = 3,
  Queen = 4,
  King = 5,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Piece {
  class: PieceType,
  color: PieceColor,
}

const WP: Piece = Piece {
  class: PieceType::Pawn,
  color: PieceColor::White,
};
const BP: Piece = Piece {
  class: PieceType::Pawn,
  color: PieceColor::Black,
};

const WKN: Piece = Piece {
  class: PieceType::Knight,
  color: PieceColor::White,
};
const BKN: Piece = Piece {
  class: PieceType::Knight,
  color: PieceColor::Black,
};

const WB: Piece = Piece {
  class: PieceType::Bishop,
  color: PieceColor::White,
};
const BB: Piece = Piece {
  class: PieceType::Bishop,
  color: PieceColor::Black,
};

const WR: Piece = Piece {
  class: PieceType::Rook,
  color: PieceColor::White,
};
const BR: Piece = Piece {
  class: PieceType::Rook,
  color: PieceColor::Black,
};

const WQ: Piece = Piece {
  class: PieceType::Queen,
  color: PieceColor::White,
};
const BQ: Piece = Piece {
  class: PieceType::Queen,
  color: PieceColor::Black,
};

const WK: Piece = Piece {
  class: PieceType::King,
  color: PieceColor::White,
};
const BK: Piece = Piece {
  class: PieceType::King,
  color: PieceColor::Black,
};

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
    board[8 * 7 + 1] = Some(WKN);
    board[8 * 7 + 6] = Some(WKN);
    board[8 * 0 + 1] = Some(BKN);
    board[8 * 0 + 6] = Some(BKN);
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

fn sort2<T: Copy + Ord>(x: T, y: T) -> (T, T) {
  if x < y {
    (x, y)
  } else {
    (y, x)
  }
}

fn is_move_legal(board: &Board, (x1, y1): (u32, u32), (x2, y2): (u32, u32)) -> bool {
  if (x1, y1) == (x2, y2) {
    return false;
  }

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
          // sanity check
          if captured_piece.color != piece.color {
            (x1 as i32 - x2 as i32).abs() == 1 && y_dist() == direction
          } else {
            false
          }
        } else if y1 == rank2 {
          x1 == x2 && file_range.contains(&y_dist())
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
      PieceType::Bishop => {
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
      PieceType::Rook => {
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
      PieceType::Queen => false,
      PieceType::King => false,
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
  let mut selection: Option<(u32, u32)> = None;
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

          if let Some((ox, oy)) = selection {
            if let Some(old_piece) = board[(ox, oy)] {
              let old_color = old_piece.color;

              let new_piece_isnt_same_color =
                || board[(x, y)].map(|p| p.color != old_color).unwrap_or(true);

              if (ox, oy) != (x, y) && new_piece_isnt_same_color() {
                println!("Legal? {}", is_move_legal(&board, (ox, oy), (x, y)));

                // move piece
                board[(x, y)] = board[(ox, oy)];
                board[(ox, oy)] = None;

                to_move = !to_move;
                println!("{:?}", to_move);
              }
            }
            selection = None;
          } else {
            // don't allow selecting empty squares
            if let Some(piece) = board[(x, y)] {
              // only allow selecting color to move
              if piece.color == to_move {
                selection = Some((x, y));
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

    board.draw(&mut window, &texture_map);

    window.display()
  }
}

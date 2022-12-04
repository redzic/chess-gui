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
      board[8 * j + 1] = Some(BP);
      board[8 * j + 6] = Some(WP);
    }

    board[8 * 0 + 7] = Some(WR);
    board[8 * 7 + 7] = Some(WR);
    board[8 * 0 + 0] = Some(BR);
    board[8 * 7 + 0] = Some(BR);
    board[8 * 1 + 7] = Some(WKN);
    board[8 * 6 + 7] = Some(WKN);
    board[8 * 1 + 0] = Some(BKN);
    board[8 * 6 + 0] = Some(BKN);
    board[8 * 2 + 7] = Some(WB);
    board[8 * 5 + 7] = Some(WB);
    board[8 * 2 + 0] = Some(BB);
    board[8 * 5 + 0] = Some(BB);
    board[8 * 3 + 0] = Some(BQ);
    board[8 * 3 + 7] = Some(WQ);
    board[8 * 4 + 0] = Some(BK);
    board[8 * 4 + 7] = Some(WK);

    Self { board }
  }

  fn draw(&self, window: &mut RenderWindow, texture_map: &[SfBox<Texture>; 12]) {
    for i in 0..8 {
      for j in 0..8 {
        if let Some(piece) = self.board[8 * i + j] {
          piece.draw((i as u32, j as u32), window, texture_map);
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

          println!("click: ({}, {})", x, y);

          if let Some((ox, oy)) = selection {
            if let Some(old_piece) = board[8 * ox + oy] {
              let old_color = old_piece.color;

              let new_piece_isnt_same_color = || {
                board[8 * x + y]
                  .map(|p| p.color != old_color)
                  .unwrap_or(true)
              };

              if (ox, oy) != (x, y) && new_piece_isnt_same_color() {
                // move piece
                board[8 * x + y] = board[8 * ox + oy];
                board[8 * ox + oy] = None;
              }
            }
            selection = None;
          } else {
            // don't allow selecting empty squares
            if board[8 * x + y] != None {
              selection = Some((x, y));
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
        if ((i & 1) ^ (j & 1)) != 0 {
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

use sfml::graphics::{
  Color, RectangleShape, RenderTarget, RenderWindow, Shape, Sprite, Texture, Transformable,
};
use sfml::system::{Vector2, Vector2f};
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

#[derive(Copy, Clone)]
enum PieceColor {
  White = 0,
  Black = 1,
}

#[derive(Copy, Clone)]
enum PieceType {
  Pawn = 0,
  Knight = 1,
  Bishop = 2,
  Rook = 3,
  Queen = 4,
  King = 5,
}

#[derive(Copy, Clone)]
struct Piece {
  class: PieceType,
  color: PieceColor,
}

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

  loop {
    while let Some(event) = window.poll_event() {
      match event {
        Event::Closed
        | Event::KeyPressed {
          code: Key::Escape, ..
        } => return,
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

    for i in 0..8 {
      for j in 0..8 {
        Piece {
          class: PieceType::King,
          color: PieceColor::Black,
        }
        .draw((i, j), &mut window, &texture_map);
      }
    }

    window.display()
  }
}

use crate::*;

pub fn worst_eval(color: PieceColor) -> i32 {
  if color.is_white() {
    -1_000_000
  } else {
    1_000_000
  }
}

pub fn minimax(
  board: Board,
  depth: u32,
  color: PieceColor,
  mut alpha: i32,
  mut beta: i32,
) -> (Option<Move>, i32) {
  if depth == 0 {
    return (None, board.eval(color));
  }

  let moves = board.moves_for_player(color);

  // maximizing player
  if color.is_white() {
    let mut best_val = i32::MIN;
    let mut best_move = None;
    for mv in moves {
      let (_, value) = minimax(board.apply_move(mv), depth - 1, !color, alpha, beta);

      if value > best_val {
        best_move = Some(mv);
      }

      best_val = i32::max(best_val, value);
      alpha = i32::max(alpha, best_val);

      if beta <= alpha {
        break;
      }
    }
    return (best_move, best_val);
  } else {
    let mut best_val = i32::MAX;
    let mut best_move = None;
    for mv in moves {
      let (_, value) = minimax(board.apply_move(mv), depth - 1, !color, alpha, beta);

      if value < best_val {
        best_move = Some(mv);
      }

      best_val = i32::min(best_val, value);
      beta = i32::min(beta, best_val);

      if beta <= alpha {
        break;
      }
    }
    return (best_move, best_val);
  }
}

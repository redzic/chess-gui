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
  alpha: i32,
  beta: i32,
) -> (Option<Move>, i32) {
  let choose = if color.is_white() { i32::max } else { i32::min };

  if depth == 0 {
    let moves = board.moves_for_player(color);

    if moves.is_empty() {
      let eval = if color.is_white() {
        -1_000_000
      } else {
        1_000_000
      };
      return (None, eval);
    }

    let mut best_eval = worst_eval(color);
    let mut best_move = None;

    for mv in moves {
      let mv_eval = board.apply_move(mv).eval(!color);
      if choose(mv_eval, best_eval) == mv_eval {
        best_eval = mv_eval;
        best_move = Some(mv);
      }
    }

    (best_move, best_eval)
  } else {
    let moves = board.moves_for_player(color);

    if moves.is_empty() {
      return (None, worst_eval(color));
    }

    let mut best_eval = worst_eval(color);
    let mut best_move = None;

    for mv in moves {
      let (_, mv_eval) = minimax(board.apply_move(mv), depth - 1, !color, alpha, beta);

      if choose(mv_eval, best_eval) == mv_eval {
        best_eval = mv_eval;
        best_move = Some(mv);
      }
    }

    (best_move, best_eval)
  }
}

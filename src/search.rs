use crate::*;

// minimax search (no alpha-beta pruning yet)
pub fn minimax(board: Board, depth: u32, color: PieceColor) -> (Option<Move>, i32) {
  let choose = if color.is_white() { i32::max } else { i32::min };
  //   let choose = i32::min;

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

    // TODO handle this somehow
    assert!(!moves.is_empty());

    let best_move = moves
      .iter()
      .reduce(|a, b| {
        let a_eval = board.apply_move(*a).eval(!color);
        let b_eval = board.apply_move(*b).eval(!color);

        if choose(a_eval, b_eval) == a_eval {
          a
        } else {
          b
        }
      })
      .unwrap();

    // TODO: redundant evaluation
    (Some(*best_move), board.apply_move(*best_move).eval(!color))
  } else {
    // assert!(depth == 1);

    let mut moves: Vec<(Move, i32)> = board
      .moves_for_player(color)
      .iter()
      .map(|mv| (*mv, 0))
      .collect();

    for (mv, eval_to_update) in &mut moves {
      let (_, mv_eval) = minimax(board.apply_move(*mv), depth - 1, !color);
      *eval_to_update = mv_eval;
    }

    // now choose best move

    let (best_move, eval) = moves
      .iter()
      .reduce(|a, b| if choose(a.1, b.1) == a.1 { a } else { b })
      .unwrap();

    (Some(*best_move), *eval)
  }
}

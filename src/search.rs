use shakmaty::{Chess, Move, Position};
use shakmaty_uci::{UciMessage, UciTimeControl};

use crate::eval::{evaluate};

pub struct ChessMove {
    score: i32,
    chess_move: Option<Move>
}

impl ChessMove{
    fn new(score: i32, chess_move: Option<Move>) -> Self {
        ChessMove { score, chess_move }
    }
}

pub fn get_best_move(board: &Chess, depth: u32, time_control: Option<UciTimeControl>) -> (Option<Move>, Option<Move>) {
    let mut best_move: Option<Move> = None;
    let mut opponent_best_answer: Option<Move> = None;
    let mut best_score: i32 = -i32::MAX;

    let possible_moves = board.legal_moves();

    if possible_moves.is_empty() {
        return (None, None)
    }

    for m in possible_moves {
        let mut cloned_pos = board.clone();
        cloned_pos.play_unchecked(m); // now opponents turn

        let alpha_beta_result = alpha_beta(&cloned_pos, -i32::MAX, i32::MAX, depth - 1);
        let score = -alpha_beta_result.score; // score is returned from opponents perspective (positive for them = negative for me)

        if score > best_score {
            best_score = score;
            best_move = Some(m);
            opponent_best_answer = alpha_beta_result.chess_move;
        }
    }
    (best_move, opponent_best_answer)
}

// ich bin ein mastermind
// alpha beta pruning works by minimizing the bounds on the worst score
// the player whos turn it is will get and the best score the player whos turn it inst it will allow
//
// nur hälfte gecheckt aber egal

pub fn alpha_beta(board: &Chess, mut alpha: i32, beta: i32, depth: u32) -> ChessMove {
    let possible_moves = board.legal_moves();

    if possible_moves.is_empty() {
        if board.is_checkmate() { // is the current player getting checkmated
            return ChessMove::new(-200000 - depth as i32, None) // apply huge debuff
        }
        return ChessMove::new(0, None);
    }

    if depth == 0 {
        return ChessMove::new(evaluate(board, board.turn()), None) // eval current player
    }

    let mut best_opponent_move_at_node: Option<Move> = None;

    for m in possible_moves {
        let mut cloned_pos = board.clone();
        cloned_pos.play_unchecked(m);
        // aplha = the lower bound the maximizing player is guaranteed to get
        // beta = the upper bound on the score the minimizing player is to allow
        let score = -alpha_beta(&cloned_pos, -beta, -alpha, depth - 1).score; // opponents turn =>
        if score >= beta {
            return ChessMove::new(beta, Some(m));
            
        }
        if score > alpha {
            alpha = score;
            best_opponent_move_at_node = Some(m);
        }
    }
    ChessMove::new(alpha, best_opponent_move_at_node)
}

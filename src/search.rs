use shakmaty::{Chess, Move, Position};

use crate::eval::{evaluate};

pub fn get_best_move(board: &Chess, depth: u32) -> Option<Move> {
    let mut best_move: Option<Move> = None;
    let mut best_score: i32 = -i32::MAX;

    let possible_moves = board.legal_moves();

    if possible_moves.is_empty() {
        return None
    }

    for m in possible_moves {
        let mut cloned_pos = board.clone();
        cloned_pos.play_unchecked(m); // now opponents turn

        let score = -alpha_beta(&cloned_pos, -i32::MAX, i32::MAX, depth - 1); // score is returned from opponents perspective (positive for them = negative for me)

        if score > best_score {
            best_score = score;
            best_move = Some(m);
        }
    }
    best_move
}

// ich bin ein mastermind
// alpha beta pruning works by minimizing the bounds on the worst score
// the player whos turn it is will get and the best score the player whos turn it inst it will allow
//
// nur hälfte gecheckt aber egal

pub fn alpha_beta(board: &Chess, mut alpha: i32, beta: i32, depth: u32) -> i32 {
    let possible_moves = board.legal_moves();

    if possible_moves.is_empty() {
        if board.is_checkmate() { // is the current player getting checkmated
            return -200000 - depth as i32 // apply huge debuff
        }
        return 0;
    }

    if depth == 0 {
        return evaluate(board, board.turn()) // eval current player
    }

    for m in possible_moves {
        let mut cloned_pos = board.clone();
        cloned_pos.play_unchecked(m);
        // aplha = the lower bound the maximizing player is guaranteed to get
        // beta = the upper bound on the score the minimizing player is to allow
        let score = -alpha_beta(&cloned_pos, -beta, -alpha, depth - 1); // opponents turn =>
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score
        }
    }
    alpha
}

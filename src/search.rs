use std::time::{Duration, Instant};

use shakmaty::{Chess, Color::{self, Black, White}, Move, Position};
use shakmaty_uci::{UciMessage, UciTimeControl};

use crate::{eval::evaluate, psqt::piece_value};

pub struct ChessMove {
    score: i32,
    chess_move: Option<Move>
}

impl ChessMove{
    fn new(score: i32, chess_move: Option<Move>) -> Self {
        ChessMove { score, chess_move }
    }
}

pub struct SearchManager {
    deadline: Option<Instant>,
    nodes_searched: u32,
    stopped: bool
}

impl SearchManager {
    fn new(deadline: Option<Instant>) -> Self {
        SearchManager{deadline, nodes_searched: 0, stopped: false}
    }
    fn should_stop(&mut self) -> bool {
        self.nodes_searched += 1;
        if let Some(deadline) = self.deadline {
            // evers 2048 it gets checked if the move time is over
            if self.nodes_searched % 2048 == 0 {
                if Instant::now() >= deadline {
                    self.stopped = true
                }
            }
        }
        self.stopped
    }
}

pub fn calc_time_budget(
    white_time: Option<Duration>,
    black_time: Option<Duration>,
    white_increment: Option<Duration>,
    black_increment: Option<Duration>,
    moves_to_go: Option<u8>,
    playing_color: &Color
) -> Duration {
    let (time, increment) = match playing_color {
        White => {(white_time, white_increment)}
        Black => {(black_time, black_increment)}
    };
    let time = time.unwrap_or_default();
    let increment = increment.unwrap_or_default(); // default increment is 0 (hopefully)
    let moves_to_go = moves_to_go.unwrap_or(30);

    let time_budget = time / moves_to_go as u32 + increment / 2;

    time_budget
}

pub fn get_move_value(m: &Move) -> i32 {
    let mut score = 0;
    if let Some(victim) = m.capture() {
        score = 10 * piece_value(victim) - piece_value(m.role());
    }
    if m.promotion().is_some() { score += 9000; }
    score
}

pub fn get_best_move(
    board: &Chess,
    depth: u32,
    time_control: Option<UciTimeControl>,
    playing_color: &Color
) -> (Option<Move>, Option<Move>) {
    let possible_moves = board.legal_moves();
        if possible_moves.is_empty() {
            return (None, None);
        }

    let mut move_time: Option<Duration> = None;

    if let Some(time_control) = time_control {
        match time_control {
            UciTimeControl::MoveTime(duration) => { move_time = Some(duration.saturating_sub(Duration::from_millis(25))) }
            UciTimeControl::TimeLeft { white_time, black_time, white_increment, black_increment, moves_to_go } => {
                move_time = Some(calc_time_budget(white_time, black_time, white_increment, black_increment, moves_to_go, playing_color));
            }
            _ => {} // ponder or infinite time => no time limits apply
        }
    }

    let time_deadline = move_time.map(|t| Instant::now() + t); // map ist hella aura: macht hier sogar sinn weil das ja beides options sind und ich sozusagen einfach hinmappe halt idk wie ich es erklären soll
    let mut search_manager = SearchManager::new(time_deadline);

    let mut result = (Some(possible_moves[0]), None); // fallback if smt doesnt just pick erstbester (was ist das englische wort) move

    for d in 1..=depth { // iterative deepening
        let this_iteration = search_moves(board, d, &mut search_manager);
        if search_manager.stopped { break; }
        result = this_iteration;
    }

    result
}

pub fn search_moves(board: &Chess, depth: u32, search_manager: &mut SearchManager) -> (Option<Move>, Option<Move>) {
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

        let alpha_beta_result = alpha_beta(&cloned_pos, -i32::MAX, -best_score, depth - 1, search_manager);
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

pub fn alpha_beta(board: &Chess, mut alpha: i32, beta: i32, depth: u32, search_manager: &mut SearchManager) -> ChessMove {
    if search_manager.should_stop() {
        return ChessMove::new(0, None);   // value is garbage bc it stopped midway. caller thros it away
    }

    let mut possible_moves = board.legal_moves();

    if possible_moves.is_empty() {
        if board.is_checkmate() { // is the current player getting checkmated
            return ChessMove::new(-200000 - depth as i32, None) // apply huge debuff
        }
        return ChessMove::new(0, None);
    }

    if board.halfmoves() >= 100 { // 50 move rule
        return ChessMove::new(0, None);
    }


    if depth == 0 && !board.is_check() {
        // trigger quienscne when doing "nothing"
        let stand_pat = evaluate(board, board.turn());
        // if doing nothing is atleast beta the opponent wouldnt have allowed it. dont pursue the move further
        if stand_pat >= beta {
            return ChessMove::new(stand_pat, None)
        }
        // if dooing nothing improves current best guarante for us make it new baseline => new captures have to beat that baseline
        if stand_pat >= alpha {
            alpha = stand_pat
        }
        // remove every move thats a quietet position so only captures and promotions get evaluated (without depth)
        for i in 0..possible_moves.len() {
            if !possible_moves[i].is_capture() || !possible_moves[i].is_conversion() {
                possible_moves.remove(i);
            }
        }
    }

    let mut best_opponent_move_at_node: Option<Move> = None;

    // sort move by highest capture first
    possible_moves.sort_unstable_by_key(|m| -get_move_value(m));

    for m in possible_moves {
        let mut cloned_pos = board.clone();
        cloned_pos.play_unchecked(m);
        // aplha = the lower bound the maximizing player is guaranteed to get
        // beta = the upper bound on the score the minimizing player is to allow
        let score = -alpha_beta(&cloned_pos, -beta, -alpha, depth - 1, search_manager).score; // opponents turn =>
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

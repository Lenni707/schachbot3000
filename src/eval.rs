use shakmaty::{Chess, Color, Role, Position};
use crate::psqt::*;

pub fn evaluate(pos: &Chess, color_to_eval: Color) -> i32 {
    let mut score = 0;

    let val_pawns   = eval_pieces(pos, Role::Pawn,   PAWNS_PST,   PAWN_VAL, color_to_eval);
    let val_knights = eval_pieces(pos, Role::Knight, KNIGHT_PST,  KNIGHT_VAL, color_to_eval);
    let val_bishops = eval_pieces(pos, Role::Bishop, BISHOP_PST,  BISHOP_VAL, color_to_eval);
    let val_rooks   = eval_pieces(pos, Role::Rook,   ROOK_PST,    ROOK_VAL, color_to_eval);
    let val_queens  = eval_pieces(pos, Role::Queen,  QUEEN_PST,   QUEEN_VAL, color_to_eval);
    let val_kings   = eval_pieces(pos, Role::King,   KING_PST,    KING_VAL, color_to_eval);

    score += val_pawns + val_knights + val_bishops + val_rooks + val_queens + val_kings;

    score
}

pub fn eval_pieces(pos: &Chess, piece: Role, pst_piece: [i32; 64], val_piece: i32, color_to_eval: Color) -> i32 { // evals based on whos turn it is rn
    let mut score = 0;

    let pieces = pos.board().by_role(piece);

    for square in pieces {
        let piece = pos.board().piece_at(square).unwrap();
        // check where the piece is
        let val = if piece.color == Color::White {
            pst_piece[square.flip_vertical() as usize]
        } else {
            pst_piece[square as usize]
        };

        if piece.color == color_to_eval {
            score += val + val_piece;
        } else {
            score -= val + val_piece;
        }
    }

    score
}

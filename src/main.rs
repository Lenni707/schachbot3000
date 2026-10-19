use shakmaty::{Move, Position, Chess};

fn main() {
    let pos = Chess::default();
    let legals = pos.legal_moves();
    println!("{:?}", legals)
}

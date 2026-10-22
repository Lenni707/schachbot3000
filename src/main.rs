use std::io::{self, BufRead, Write};
use shakmaty::{CastlingMode, Chess, Position};
use shakmaty_uci::{UciMessage, UciMove, UciTimeControl};

mod eval;
mod psqt;
mod search;

use crate::search::get_best_move;

const DEPTH: u32 = 5;

// halbe vibecode weil die wichs docs alle richtig kacke waren
fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut current_pos = Chess::default();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // parse the input got from the gui
        if let Ok(msg) = trimmed.parse::<UciMessage>() {
            match msg {
                UciMessage::Uci => {
                    writeln!(stdout, "id name Schachbot3000")?;
                    writeln!(stdout, "id author Boss")?;
                    writeln!(stdout, "uciok")?;
                    stdout.flush()?;
                }
                UciMessage::IsReady => {
                    writeln!(stdout, "readyok")?;
                    stdout.flush()?;
                }
                UciMessage::Position { startpos, fen, moves } => {
                    if let Some(fen_pos) = fen {
                        current_pos = fen_pos.into_position(CastlingMode::Standard).expect("conversion from fen to chess failed");
                    } else if startpos {
                        current_pos = Chess::default();
                    }

                    // replay played moves onto position if i get a set of moves
                    for uci_m in moves {
                        if let Ok(m) = uci_m.to_move(&current_pos) {
                            if let Ok(next) = current_pos.clone().play(m) {
                                current_pos = next;
                            }
                        }
                    }
                }
                UciMessage::Go { .. } => { // TODO ADD TIME BASED DECISSIONS
                    let best_shakmaty_move = get_best_move(&current_pos, DEPTH);

                    let uci_move = UciMove::from_move(best_shakmaty_move.expect("no valid moves left"), CastlingMode::Standard); // .unwrap fuckt todes ab aber ka wie ich es mach soll

                    let response = UciMessage::BestMove {
                        best_move: uci_move,
                        ponder: None,
                    };

                    writeln!(stdout, "{}", response)?;
                    stdout.flush()?;
                }
                UciMessage::Quit => break,
                _ => {}
            }
        }
    }

    Ok(())
}
// TODO FOR EVAL:
// ALREAFY COMPLETED (SORT OF)
//      - PST TABLE
//      - INDIVDUAL VALUE OF PIECES
// TODO
//      - ATTACKER/DEFENDER
//      - PIECE SPECIFICS (DOUBLE ROCKS, DOUBLE PAWNS PUNISHED, ETC...)
//
// TODO FOR LOOP
// TIME MANAHEDMENT
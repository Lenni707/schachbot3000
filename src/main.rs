use std::io::{self, BufRead, Write};
use std::{thread, time};
use std::sync::mpsc;
use shakmaty::{CastlingMode, Chess, Move, Position};
use shakmaty_uci::{UciMessage, UciMove, UciTimeControl};

mod eval;
mod psqt;
mod search;

use crate::search::get_best_move;

const DEPTH: u32 = 64;

fn main() -> io::Result<()> {
    let (tx, rx) = mpsc::channel::<UciMessage>(); // one sender and one receiver channel

    thread::spawn(move || -> io::Result<()> { // thread to handle the output from the gui
        let stdin = io::stdin();

        for line in stdin.lines() {
            let line = line?; // kb mehr
            if let Ok(msg) = line.trim().parse::<UciMessage>() { // TODES GEIL ::<> aka "turbofish" zeigt genrischer function den return type
                if tx.send(msg).is_err() { break; }
            }

        }
        Ok(())
    });
    // main thread
    let mut stdout = io::stdout();
    let mut current_pos = Chess::default();

    for received_msg in &rx {
        match received_msg {
            UciMessage::Uci => {
                writeln!(stdout, "id name Schachbot3000")?;
                writeln!(stdout, "id author Lennart G")?;
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
            UciMessage::Go { time_control, .. } => { // TODO ADD TIME BASED DECISSIONS
                let best_shakmaty_move = get_best_move(&current_pos, DEPTH, time_control, &current_pos.turn());

                let uci_move = UciMove::from_move(best_shakmaty_move.0.expect("no valid moves left, idk why game didnt end earlier"), CastlingMode::Standard); // .unwrap fuckt todes ab aber ka wie ich es mach soll
                let ponder = best_shakmaty_move.1.map(|m| UciMove::from_move(m, CastlingMode::Standard));
                let response = UciMessage::BestMove {
                    best_move: uci_move,
                    ponder
                };

                writeln!(stdout, "{}", response)?;
                stdout.flush()?;
            }
            UciMessage::PonderHit => {
                // TODO have to think of a better strategy after ponder hit
            }
            UciMessage::Quit => break,
            _ => {}
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

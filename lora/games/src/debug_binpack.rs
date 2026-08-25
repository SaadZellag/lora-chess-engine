use cozy_chess::{Board, Move, Square};
use games::binpack::{BinPackWriter, GameResult, TrainingEntry};
use engine::Eval;
use std::fs::File;

fn main() -> anyhow::Result<()> {
    // White just lost the queen - should be losing badly
    let fen = "rnbqkbn1/ppp2p1r/3p2p1/4p3/4P3/3B4/PPPP1PPP/RNB1K1NR w KQq - 0 2";
    let board: Board = fen.parse().map_err(|e| anyhow::anyhow!("{:?}", e))?;
    
    // Stockfish eval: -760 from side-to-move (white's) perspective
    // Negative = white is losing
    let stockfish_eval = -760;
    
    // Game result: White lost this game (from WHITE's perspective)
    let game_result = GameResult::BlackWins;
    
    println!("Creating debug binpack entry:");
    println!("  Board: {}", board);
    println!("  Side to move: {:?}", board.side_to_move());
    println!("  Stockfish eval (side-to-move perspective): {}", stockfish_eval);
    println!("  Game result (white's perspective): {:?}", game_result);
    
    let entry = TrainingEntry {
        board: board.clone(),
        move_played: Move {
            from: Square::D3,
            to: Square::E4,
            promotion: None,
        },
        eval: Eval::CentiPawn(stockfish_eval),
        ply: 3,
        result: game_result,
    };
    
    // Write to binpack using the proper writer
    let file = File::create("../../training-data/debug_entry.binpack")?;
    let mut writer = BinPackWriter::new(file)?;

    for _ in 0..1000 {
        writer.write_entry(&entry)?;
    }
    
    println!("\n✓ Wrote ../../training-data/debug_entry.binpack");
    
    Ok(())
}

use std::{io::{Read, Seek}, mem::MaybeUninit, str::FromStr};

use chess::{Board, ChessMove, Square};
use engine::Eval;
use sfbinpack::{CompressedReaderError, CompressedTrainingDataEntryReader, chess::r#move::Move};

#[derive(Debug, Clone, Copy)]
pub enum GameResult {
    Win,
    Loss,
    Draw,
}

#[derive(Debug, Clone, Copy)]
pub struct TrainingEntry {
    board: Board,
    move_played: ChessMove,
    eval: Eval,
    ply: u16,
    result: GameResult,
}

pub struct BinPackReader<T: Read + Seek> {
    reader: CompressedTrainingDataEntryReader<T>
}

pub struct BinPackWriter {

}

impl<T: Read + Seek> BinPackReader<T> {
    pub fn new(reader: T) -> Result<Self, CompressedReaderError> {
        let reader = CompressedTrainingDataEntryReader::new(reader);
        reader.map(|reader| Self { reader })
    }

    pub fn get_next_entry(&mut self) -> anyhow::Result<TrainingEntry> {
        if !self.reader.has_next() {
            anyhow::bail!("No more entries available");
        }


        let entry = self.reader.next();

        Ok(unsafe { MaybeUninit::zeroed().assume_init() })

        // let fen = entry.pos.fen().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        // let board = Board::from_str(&fen).map_err(|e| anyhow::anyhow!("{e:?}"))?;


        // let mv_uci: String = entry.mv.as_uci();

        // let from_square = Square::from_str(&mv_uci[0..2]).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        // let to_square = Square::from_str(&mv_uci[2..4]).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        // let promotion = if mv_uci.len() == 5 {
        //     match &mv_uci[4..5] {
        //         "q" => Some(chess::Piece::Queen),
        //         "r" => Some(chess::Piece::Rook),
        //         "b" => Some(chess::Piece::Bishop),
        //         "n" => Some(chess::Piece::Knight),
        //         _ => None,
        //     }
        // } else {
        //     None
        // };

        // let result = match entry.result {
        //         1 => GameResult::Win,
        //         0 => GameResult::Draw,
        //         -1 => GameResult::Loss,
        //         _ => anyhow::bail!("Invalid game result value"),
        // };


        // let mv = ChessMove::new(from_square, to_square, promotion);

        // Ok(TrainingEntry {
        //     board,
        //     move_played: mv,
        //     eval: stock_fish_eval_to_eval(entry.score),
        //     ply: entry.ply,
        //     result: result
            
        // })
    }

    pub fn get_next_entries(&mut self, count: usize) -> anyhow::Result<Vec<TrainingEntry>> {
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            if !self.reader.has_next() {
                break;
            }
            let entry = self.get_next_entry()?;
            entries.push(entry);
        }
        Ok(entries)
    }
}

fn stock_fish_eval_to_eval(score: i16) -> Eval {

    match score {
        -32000..=-31000 => return Eval::MatedIn((score + 32000).try_into().unwrap()),
        31000..=32000 => return Eval::MateIn((32000 - score).try_into().unwrap()),
        eval => Eval::CentiPawn(eval.into())
    }

}

fn eval_to_stock_fish_eval(eval: Eval) -> i16 {
    match eval {
        Eval::MateIn(ply) => 32000 + ply as i16,
        Eval::MatedIn(ply) => -32000 + ply as i16,
        Eval::CentiPawn(cp) => cp as i16,
    }
}
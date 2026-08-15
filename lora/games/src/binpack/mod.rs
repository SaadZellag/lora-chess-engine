use std::{fmt::Debug, io::{Read, Seek, Write}, str::FromStr};

use chess::{Board, ChessMove, Square};
use engine::Eval;
use sfbinpack::{CompressedReaderError, CompressedTrainingDataEntryReader, CompressedTrainingDataEntryWriter, CompressedWriterError, TrainingDataEntry, chess::r#move::{Move, MoveType}};

use crate::binpack::convert::{chess_piece_to_sbin_piece, chess_square_to_sbin_square, eval_to_stockfish_eval, stockfish_eval_to_eval};

mod convert;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
}

#[derive(Debug, Clone, Copy)]
pub struct TrainingEntry {
    pub board: Board,
    pub move_played: ChessMove,
    pub eval: Eval,
    pub ply: u16,
    pub result: GameResult,
}

#[derive(Debug, Clone)]
pub struct GameEntry {
    pub startpos: Board,
    pub startply: u16,
    pub moves: Vec<(ChessMove, Eval)>,
    pub result: GameResult,
}

pub struct BinPackReader<T: Read + Seek> {
    reader: CompressedTrainingDataEntryReader<T>
}

pub struct BinPackWriter<T: Write> {
    writer: CompressedTrainingDataEntryWriter<T>
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

        


        let fen = entry.pos.fen().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let board = Board::from_str(&fen).map_err(|e| anyhow::anyhow!("{e:?}"))?;


        let mv_uci: String = entry.mv.as_uci();

        let from_square = Square::from_str(&mv_uci[0..2]).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let to_square = Square::from_str(&mv_uci[2..4]).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let promotion = if mv_uci.len() == 5 {
            match &mv_uci[4..5] {
                "q" => Some(chess::Piece::Queen),
                "r" => Some(chess::Piece::Rook),
                "b" => Some(chess::Piece::Bishop),
                "n" => Some(chess::Piece::Knight),
                _ => None,
            }
        } else {
            None
        };

        let result = match entry.result {
                1 => GameResult::WhiteWins,
                0 => GameResult::Draw,
                -1 => GameResult::BlackWins,
                _ => anyhow::bail!("Invalid game result value"),
        };


        let mv = ChessMove::new(from_square, to_square, promotion);

        Ok(TrainingEntry {
            board,
            move_played: mv,
            eval: stockfish_eval_to_eval(entry.score),
            ply: entry.ply,
            result: result
            
        })
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

impl<T: Write> BinPackWriter<T> {
    pub fn new(writer: T) -> Result<BinPackWriter<T>, CompressedWriterError> {
        let writer = CompressedTrainingDataEntryWriter::new(writer);
        writer.map(|writer| Self { writer })
    }

    // Because writing is gonna be done much less frequently
    // Can afford to be disgustingly unoptimized
    // Might make my own binpack reader using the chess library
    pub fn write_game(&mut self, game: &GameEntry) -> anyhow::Result<()> {

        let mut board = game.startpos.clone();
        
        for (i, (mv, eval)) in game.moves.iter().enumerate() {
            let ply = game.startply + i as u16;

            let entry = TrainingEntry {
                board: board.clone(),
                move_played: *mv,
                eval: *eval,
                ply,
                result: game.result,
            };

            self.write_entry(&entry)?;

            board = board.make_move_new(*mv);
        }

        Ok(())
    }

    pub fn write_entry(&mut self, entry: &TrainingEntry) -> anyhow::Result<()> {
        let fen = format!("{}", entry.board);
        let entry_board = sfbinpack::chess::position::Position::from_fen(&fen).map_err(|e| anyhow::anyhow!("{e:?}"))?;


        let src = chess_square_to_sbin_square(entry.move_played.get_source());
        let mut dest = chess_square_to_sbin_square(entry.move_played.get_dest());
        let promotion = entry.move_played.get_promotion().map(|piece| {
            let color = entry.board.color_on(entry.move_played.get_source()).unwrap();
            chess_piece_to_sbin_piece(piece, color)
        }).unwrap_or(sfbinpack::chess::piece::Piece::NONE);

        let mv_type;
        if entry.move_played.get_promotion().is_some() {
            mv_type = MoveType::Promotion;
        } else if entry.board.piece_on(entry.move_played.get_source()) == Some(chess::Piece::King) && (entry.move_played.get_source().get_file() as i8 - entry.move_played.get_dest().get_file() as i8).abs() > 1 {
            mv_type = MoveType::Castle;
            dest = chess_square_to_sbin_square(match entry.move_played.get_dest() {
                Square::G1 => Square::H1,
                Square::C1 => Square::A1,
                Square::G8 => Square::H8,
                Square::C8 => Square::A8,
                _ => entry.move_played.get_dest(),
            })
        } else if entry.board.en_passant() == Some(entry.move_played.get_dest()) && entry.board.piece_on(entry.move_played.get_source()) == Some(chess::Piece::Pawn) {
            mv_type = MoveType::EnPassant;
        } else {
            mv_type = MoveType::Normal;
        }

        let result_move = sfbinpack::chess::r#move::Move::new(src, dest, mv_type, promotion);

        let score = eval_to_stockfish_eval(entry.eval);

        let result = match entry.result {
            GameResult::WhiteWins => 1,
            GameResult::Draw => 0,
            GameResult::BlackWins => -1,
        } * if entry.board.side_to_move() == chess::Color::White { 1 } else { -1 };

        let sbin_entry = sfbinpack::TrainingDataEntry {
            pos: entry_board,
            mv: result_move,
            score,
            ply: entry.ply,
            result,
        };

        // println!("{} {} {:?} {:?}", fen, entry.move_played, result_move, entry.move_played.get_promotion());

        self.writer.write_entry(&sbin_entry).map_err(|e| anyhow::anyhow!("{e:?}"))?;

        Ok(())
    }
}


mod tests {
    use chess::MoveGen;
use rand::Rng;
use tempfile::{NamedTempFile, tempdir, tempfile};

use super::*;
    use std::fs::File;
    use std::io::{BufReader, BufWriter};

    #[test]
    fn test_binpack_write_then_read() {
        let mut board = Board::default();
        let mut rng = rand::thread_rng();

        let mut moves = Vec::new();

        for i in 0..20 {
            let legal_moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();
            if legal_moves.is_empty() {
                break;
            }
            let mv = legal_moves[rng.gen_range(0..legal_moves.len())];
            let eval = Eval::CentiPawn(i * 10);
            moves.push((mv, eval));
            board = board.make_move_new(mv);
        }

        let game_entry = GameEntry {
            startpos: Board::default(),
            startply: 0,
            moves,
            result: GameResult::WhiteWins,
        };

        

        let dir = tempdir().unwrap();
        let path = dir.path().join("test_file.binpack");
        
        let file = File::create(&path).expect("Failed to create test file");
        let mut writer = BinPackWriter::new(file).unwrap();
        writer.write_game(&game_entry).unwrap();


        drop(writer);

        let file = File::open(&path).expect("Failed to open test file");
        let mut reader = BinPackReader::new(file).unwrap();

        let mut read_moves = Vec::new();
        while let Ok(entry) = reader.get_next_entry() {
            read_moves.push((entry.move_played, entry.eval));
        }

        assert_eq!(game_entry.moves, read_moves);
    }

    #[test]
    fn test_binpack_read_write() {
        let file_path = "/home/saad/Desktop/Programming/AI/lora-chess-engine/training-data/first-2-chunks.binpack";
        let file = File::open(file_path).expect("Failed to open binpack file");

        let estimated_entries = file.metadata().expect("Failed to get file metadata").len() * 2 / 5;

        let output_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_name = output_file.path().to_str().map(|s| s.to_string()).expect("Failed to get temp file path");

        let mut reader = BinPackReader::new(file).expect("Failed to create BinPackReader");
        let mut writer = BinPackWriter::new(output_file).expect("Failed to create BinPackWriter");

        let mut count = 0;

        while let Ok(entries) = reader.get_next_entries(1000) {
            if entries.is_empty() {
                break;
            }
            for entry in &entries {
                writer.write_entry(&entry).expect("Failed to write entry");
            }
            count += entries.len();
            println!("Processed {}/{} entries", count, estimated_entries);
        }


        let file = File::open(&file_name).expect("Failed to open temp file");
        let result_file = File::open(&file_name).expect("Failed to open original binpack file");


        let mut reader_original = BinPackReader::new(&file).expect("Failed to create BinPackReader for original file");
        let mut reader_result = BinPackReader::new(&result_file).expect("Failed to create BinPackReader for result file");

        loop {
            let original_entry = reader_original.get_next_entry();
            let result_entry = reader_result.get_next_entry();

            match (original_entry, result_entry) {
                (Ok(orig), Ok(res)) => {
                    assert_eq!(orig.board, res.board);
                    assert_eq!(orig.move_played, res.move_played);
                    assert_eq!(orig.eval, res.eval);
                    assert_eq!(orig.ply, res.ply);
                    assert_eq!(orig.result, res.result);
                }
                (Err(_), Err(_)) => break,
                _ => panic!("Mismatch in number of entries between original and result files"),
            }
        }


        let mut file = File::open(&file_name).expect("Failed to open temp file");
        let mut result_file = File::open(&file_name).expect("Failed to open original binpack file");

        let mut file_bytes = Vec::new();
        let mut result_bytes = Vec::new();

        std::io::Read::read_to_end(&mut file, &mut file_bytes).expect("Failed to read temp file");
        std::io::Read::read_to_end(&mut result_file, &mut result_bytes).expect("Failed to read result file");

        assert_eq!(file_bytes.len(), result_bytes.len(), "File sizes differ");
        assert_eq!(file_bytes, result_bytes, "File contents differ");
    }
}
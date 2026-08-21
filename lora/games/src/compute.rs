

use std::collections::HashMap;
use std::fs::File;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use cozy_chess::{Board, Color, Piece};
use crate::binpack::{BinPackReader, GameResult};
use indicatif::{ProgressBar, ProgressStyle};



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    eval_value: i32,
    mom_bucket: u32,
}


#[derive(Debug, Clone, Copy, Default)]
struct Value {
    wins: u32,
    losses: u32,
    draws: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct HistogramEntry {
    eval: i32,
    mom_bucket: u32,
    wins: u32,
    losses: u32,
    draws: u32,
}

pub fn compute(binpack_files: &[String], output_file: &str) {
    // Calculate total file size
    let total_bytes: u64 = binpack_files
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok().map(|m| m.len()))
        .sum();

    if total_bytes == 0 {
        eprintln!("No valid binpack files found");
        return;
    }

    // Create progress bar
    let progress_bar = ProgressBar::new(total_bytes);
    progress_bar.set_style(ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {binary_bytes}/{binary_total_bytes} ETA: {eta_precise}"
    ).unwrap().progress_chars("##-"));

    // Create readers for each file
    let mut readers: Vec<_> = binpack_files
        .iter()
        .filter_map(|path| {
            File::open(path)
                .ok()
                .and_then(|f| BinPackReader::new(f).ok())
        })
        .collect();

    if readers.is_empty() {
        eprintln!("No valid binpack files could be opened");
        return;
    }

    println!("Opened {} binpack files ({} bytes total)", readers.len(), total_bytes);

    // 2D Histogram: (eval_value, mom) -> (sum_result, count)
    let mut histogram: HashMap<Key, Value> = HashMap::new();
    let mut total_entries = 0u64;
    let mut active_readers = (0..readers.len()).collect::<Vec<_>>();
    let mut last_update = Instant::now();

    // Keep reading until all files exhausted (interleaved)
    while !active_readers.is_empty() {
        let mut i = 0;
        while i < active_readers.len() {
            let reader_idx = active_readers[i];
            
            if let Ok(entry) = readers[reader_idx].get_next_entry() {
                // Convert result from player to move's perspective


                let eval_value = entry.eval.value();
                let mom_bucket = mom_value(&entry.board);
                
                let key = Key { eval_value, mom_bucket };

                let histogram_entry = histogram.entry(key).or_default();

                match (entry.result, entry.board.side_to_move()) {
                    // Player to move wins
                    (GameResult::WhiteWins, Color::White) => histogram_entry.wins += 1,
                    (GameResult::BlackWins, Color::Black) => histogram_entry.wins += 1,
                    // Player to move loses
                    (GameResult::WhiteWins, Color::Black) => histogram_entry.losses += 1,
                    (GameResult::BlackWins, Color::White) => histogram_entry.losses += 1,
                    // Draw
                    (GameResult::Draw, _) => histogram_entry.draws += 1,
                };

                
                total_entries += 1;

                // Update progress bar every second
                if last_update.elapsed().as_secs() >= 1 {
                    let bytes_read = readers[reader_idx].read_bytes();
                    progress_bar.set_position(bytes_read);
                    last_update = Instant::now();
                }
                
                i += 1;
            } else {
                // This reader is exhausted, remove it
                active_readers.swap_remove(i);
            }
        }
    }

    progress_bar.finish_with_message("Done reading");
    println!("Total entries processed: {}", total_entries);
    println!("Unique (eval, mom) pairs: {}", histogram.len());

    // Convert histogram to JSON format
    let mut json_data: Vec<HistogramEntry> = histogram
        .iter()
        .map(|(key, value)| HistogramEntry {
            eval: key.eval_value,
            mom_bucket: key.mom_bucket,
            wins: value.wins,
            losses: value.losses,
            draws: value.draws,
        })
        .collect();
    
    // Sort by eval then mom_bucket for consistent output
    json_data.sort_by_key(|e| (e.eval, e.mom_bucket));
    
    // Write to file
    match std::fs::File::create(output_file) {
        Ok(file) => {
            match serde_json::to_writer_pretty(file, &json_data) {
                Ok(_) => println!("Histogram written to {}", output_file),
                Err(e) => eprintln!("Failed to serialize histogram: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to create output file: {}", e),
    }
}

fn mom_value(board: &Board) -> u32 {
   board.pieces(Piece::Queen).len() * 9
        + board.pieces(Piece::Rook).len() * 5
        + board.pieces(Piece::Bishop).len() * 3
        + board.pieces(Piece::Knight).len() * 3
        + board.pieces(Piece::Pawn).len() * 1

}

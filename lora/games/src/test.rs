use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, Mutex},
};

use common::cozy_chess::CozyChessHelper;
use cozy_chess::{Board, Color};
use engine::{Eval, EVALUATOR, NNUEAccumulator};
use features::FEATURES_PER_SIDE;
use games::binpack::{BinPackReader, BinPackWriter};
use indicatif::ProgressStyle;
use tempfile::tempdir;

pub fn test(binpack_file: &str) {
    test_binpack_read_write(binpack_file);
}

fn test_binpack_read_write(binpack_file: &str) {
    let file = std::fs::File::open(binpack_file).expect("Failed to open binpack file");

    let file_size = file.metadata().expect("Failed to get file metadata").len();
    let expected_entries = file_size * 2 / 5;

    let dir = tempdir().unwrap();
    let path = dir.path().join("test_file.binpack");
    let output_file = std::fs::File::create(&path).expect("Failed to create test file");

    let mut reader = BinPackReader::new(file).expect("Failed to create BinPackReader");
    let mut writer = BinPackWriter::new(output_file).expect("Failed to create BinPackWriter");

    let progress_bar = indicatif::ProgressBar::new(expected_entries);
    progress_bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
    .unwrap()
    .progress_chars("##-"));
    

    println!("Test file: {:?}", path);

    let chunk_size = 1000;

    while let Ok(entries) = reader.get_next_entries(chunk_size) {
        for entry in &entries {
            writer.write_entry(&entry).expect("Failed to write entry");
        }
        progress_bar.inc(entries.len() as u64);
        progress_bar.set_message(format!("ETA: {:?}", progress_bar.eta()));
    }

    progress_bar.finish_with_message("Done reading and writing entries");
}
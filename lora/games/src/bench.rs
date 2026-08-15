use std::{env::args, fs::OpenOptions, time::Instant};
use games::binpack::BinPackReader;

pub fn bench(path: &str) {

    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .append(false)
        .open(path)
        .unwrap();

    let mut reader = BinPackReader::new(file).unwrap();

    const CHUNK_SIZE: usize = 2 << 16;
    const NUM_CHUNKS: usize = 10;

    let start = Instant::now();
    let mut total_count = 0;

    for _ in 0..NUM_CHUNKS {
        let entries = reader.get_next_entries(CHUNK_SIZE).unwrap();
        total_count += entries.len();
    }

    let duration = start.elapsed();
    println!("Total entries read: {}", total_count);
    println!("Time taken: {:?}", duration);
    println!(
        "Entries per second: {:.2}",
        total_count as f64 / duration.as_secs_f64()
    );
}
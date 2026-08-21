use clap::Args;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

#[derive(Args, Clone)]
pub struct Options {
    #[arg(long, num_args = 1..)]
    binpack_files: Vec<String>,
    #[arg(long)]
    output_training: String,
    #[arg(long)]
    output_testing: String,
    #[arg(long, default_value_t = 0.8)]
    training_ratio: f32,
}

pub fn prepare_training_data(options: Options) {
    let Options {
        binpack_files,
        output_training,
        output_testing,
        training_ratio,
    } = options;

    let training_ratio = training_ratio as f64;

    let train_file = File::create(&output_training).expect("Failed to create training file");
    let test_file = File::create(&output_testing).expect("Failed to create testing file");
    let mut train_writer = BufWriter::new(train_file);
    let mut test_writer = BufWriter::new(test_file);

    let mut total_train_bytes: u64 = 0;
    let mut total_test_bytes: u64 = 0;

    let magic = b"BINP";

    for path in &binpack_files {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Warning: Failed to open {}: {}", path, e);
                continue;
            }
        };

        let mut reader = BufReader::new(file);

        loop {
            // Step 1: Read the 4-byte magic string ("BINP")
            let mut header = [0u8; 4];
            match reader.read_exact(&mut header) {
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break; // Normal EOF reached at a chunk boundary
                }
                Err(e) => {
                    eprintln!("Error reading magic header in {}: {}", path, e);
                    break;
                } // Error or EOF
            }

            if &header != magic {
                eprintln!("Corrupted chunk or invalid header in file {}", path);
                break;
            }

            // Step 2: Read 4-byte little-endian length integer
            let mut size_bytes = [0u8; 4];
            if let Err(e) = reader.read_exact(&mut size_bytes) {
                eprintln!("Unexpected EOF reading size in {}: {}", path, e);
                break;
            }
            let payload_size = u32::from_le_bytes(size_bytes) as usize;

            // Step 3: Read payload data
            let mut payload = vec![0u8; payload_size];
            if let Err(e) = reader.read_exact(&mut payload) {
                eprintln!("Unexpected EOF reading payload in {}: {}", path, e);
                break;
            }

            // Total size includes header (4) + length field (4) + payload
            let total_chunk_size = (8 + payload_size) as u64;

            // Step 4: Balance files based on byte allocation
            let grand_total = total_train_bytes + total_test_bytes;
            let send_to_train = if grand_total == 0 {
                true
            } else {
                (total_train_bytes as f64 / grand_total as f64) < training_ratio
            };

            let writer = if send_to_train {
                total_train_bytes += total_chunk_size;
                &mut train_writer
            } else {
                total_test_bytes += total_chunk_size;
                &mut test_writer
            };

            // Write full header + size + payload to target output
            writer.write_all(&header).expect("Write failed");
            writer.write_all(&size_bytes).expect("Write failed");
            writer.write_all(&payload).expect("Write failed");
        }
    }

    train_writer.flush().expect("Failed to flush training writer");
    test_writer.flush().expect("Failed to flush testing writer");

    let grand_total = total_train_bytes + total_test_bytes;
    if grand_total > 0 {
        println!("Split completed successfully!");
        println!(
            "  Training: {} bytes ({:.2}%)",
            total_train_bytes,
            (total_train_bytes as f64 / grand_total as f64) * 100.0
        );
        println!(
            "  Testing:  {} bytes ({:.2}%)",
            total_test_bytes,
            (total_test_bytes as f64 / grand_total as f64) * 100.0
        );
    }
}
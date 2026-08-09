use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use games::utils::ENTRY_SIZE_BYTES;

use crate::utils::SparseBatch;

#[derive(Debug)]
#[repr(C)]
pub struct SparseBatchStream {
    reader: BufReader<File>,
    buffer: Vec<u8>,
    num_batches: usize,
}

impl SparseBatchStream {
    pub fn new<P>(path: P, mut batch_size: usize) -> Option<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).ok()?;
        let file_size = file.metadata().ok()?.len() as usize;
        let num_batches = if file_size < (batch_size * ENTRY_SIZE_BYTES) {
            batch_size = file_size / ENTRY_SIZE_BYTES;
            1
        } else {
            file_size / (batch_size * ENTRY_SIZE_BYTES)
        };

        let reader = BufReader::new(file);

        let buffer = vec![0; batch_size * ENTRY_SIZE_BYTES];

        Some(Self {
            reader,
            buffer,
            num_batches,
        })
    }
}

impl Iterator for SparseBatchStream {
    type Item = SparseBatch;

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(_) = self.reader.read_exact(&mut self.buffer) {
            self.reader.seek(SeekFrom::Start(0)).ok()?;
        }

        let data = self
            .buffer
            .chunks(ENTRY_SIZE_BYTES)
            .map(|c| bincode::deserialize(c).unwrap())
            .collect::<Vec<_>>();

        // data.shuffle(&mut thread_rng());

        Some(SparseBatch::new(&data))
    }
}

impl ExactSizeIterator for SparseBatchStream {
    fn len(&self) -> usize {
        self.num_batches
    }
}

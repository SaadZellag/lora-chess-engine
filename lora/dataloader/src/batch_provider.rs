use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use games::{binpack::BinPackReader};

use crate::utils::SparseBatch;

#[derive(Debug)]
#[repr(C)]
pub struct SparseBatchStream {
    file_name: String,
    reader: BinPackReader<BufReader<File>>,
    batch_size: usize
}

impl SparseBatchStream {
    pub fn new<P>(path: P, batch_size: usize) -> Option<Self>
    where
        P: AsRef<Path>,
    {
        let file_name: String = path.as_ref().file_name()?.to_str()?.to_owned();
        let file = File::open(path).ok()?;

        let file = BufReader::new(file);
        let reader = BinPackReader::new(file).ok()?;


        Some(Self {
            file_name,
            reader,
            batch_size
        })
    }
}

impl Iterator for SparseBatchStream {
    type Item = SparseBatch;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.reader.has_next() {
            let file = BufReader::new(File::open(&self.file_name).ok()?);
            self.reader = BinPackReader::new(file).ok()?;
        }

        let entries = self.reader.get_next_entries(self.batch_size).ok()?;

        Some(SparseBatch::new(&entries))
    }
}
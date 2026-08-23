use std::{
    fs::File, io::{BufReader, Read, Seek, SeekFrom}, iter::Flatten, path::Path, pin::Pin,
};

use games::{binpack::BinPackReader};
use stockfish_binpack::reader::mmap::{BinPackChunk, MmapBinpackReader};

use crate::utils::SparseBatch;

#[repr(C)]
pub struct SparseBatchStream {
    file_name: String,
    reader: Pin<Box<MmapBinpackReader>>,
    iterator: Flatten<std::vec::IntoIter<BinPackChunk<'static>>>,
    batch_size: usize
}

impl SparseBatchStream {
    pub fn new<P>(path: P, batch_size: usize) -> Option<Self>
    where
        P: AsRef<Path>,
    {
        let file_name: String = path.as_ref().file_name()?.to_str()?.to_owned();
        let file = File::open(path).ok()?;

        let mmap_reader = unsafe { Box::pin(MmapBinpackReader::new(&file).ok()?) };

        let chunk_iterator: Flatten<std::vec::IntoIter<BinPackChunk<'static>>> = 
            unsafe { std::mem::transmute(
                mmap_reader.get_chunks().ok()?
                .into_iter()
                .flatten()
            ) };


        Some(Self {
            file_name,
            reader: mmap_reader,
            batch_size,
            iterator: chunk_iterator,
        })
    }
}

impl Iterator for SparseBatchStream {
    type Item = SparseBatch;

    fn next(&mut self) -> Option<Self::Item> {
        // if !self.reader.has_next() {
        //     let file = BufReader::new(File::open(&self.file_name).ok()?);
        //     self.reader = BinPackReader::new(file).ok()?;
        // }

        // let entries = self.reader.get_next_entries(self.batch_size).ok()?;
        let entries = self.iterator.by_ref().take(self.batch_size).collect::<Vec<_>>();

        Some(SparseBatch::new(&entries))
    }
}
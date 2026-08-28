use std::{fs::File, io::{BufReader, Cursor, Read, Seek, SeekFrom}, iter::Flatten, path::Path, pin::Pin,
};

use games::{binpack::BinPackReader};
use stockfish_binpack::{BPEntry, reader::{bpreader::BPChunkReader, mmap::MmapBinpackReader}};

use crate::{batch::SparseBatch, stream::SparseBatchStream};

#[repr(C)]
pub struct SparseBatchStreamProvider {
    reader: MmapBinpackReader,
    batch_size: usize,
    num_workers: usize,
    step_size: usize,
}

impl SparseBatchStreamProvider {
    pub fn new<P>(path: P, batch_size: usize, num_workers: usize, step_size: usize) -> Option<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).ok()?;

        let mmap_reader = unsafe { MmapBinpackReader::new(&file).ok()? };

        Some(Self {
            reader: mmap_reader,
            batch_size,
            num_workers,
            step_size,
        })
    }

    pub fn get_stream(&self, worker_id: usize, current_epoch: usize) -> SparseBatchStream {
        let chunks = self.reader.get_chunks();
        let chunks: Vec<BPChunkReader<Cursor<&'static [u8]>>> = unsafe { std::mem::transmute(chunks) };

        let chunks_for_worker = chunks.into_iter().skip(worker_id).step_by(self.num_workers);
        let reader = chunks_for_worker.flatten().skip(current_epoch).step_by(self.step_size);
        SparseBatchStream::new(reader, self.batch_size)
    }
}
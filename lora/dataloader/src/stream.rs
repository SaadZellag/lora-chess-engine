use std::{io::Cursor, iter::{Flatten, Skip, StepBy}, vec::IntoIter};

use stockfish_binpack::{BPEntry, reader::bpreader::BPChunkReader};
use crate::batch::SparseBatch;

type StreamIterator = StepBy<Skip<Flatten<StepBy<Skip<IntoIter<BPChunkReader<Cursor<&'static [u8]>>>>>>>>;


#[repr(C)]
pub struct SparseBatchStream {
    reader: StreamIterator,
    batch_size: usize,
}

impl SparseBatchStream {
    pub fn new(reader: StreamIterator, batch_size: usize) -> Self {
        Self {
            reader: reader,
            batch_size,
        }
    }
}

impl Iterator for SparseBatchStream {
    type Item = SparseBatch;

    fn next(&mut self) -> Option<Self::Item> {
        let entries = self.reader.by_ref().take(self.batch_size).collect::<Vec<_>>();
        (!entries.is_empty()).then(|| SparseBatch::new(&entries))
    }
}
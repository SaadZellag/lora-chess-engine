use crate::search::position::Position;
use chess::ChessMove;

use crate::Eval;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TTEntry {
    pub hash: u64,
    pub flag: EntryType,
    pub depth: u8,
    pub eval: Eval,
    pub mv: ChessMove,
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            hash: 0,
            flag: EntryType::Invalid,
            depth: 0,
            eval: Eval::NEUTRAL,
            mv: ChessMove::default(),
        }
    }
}

pub struct TranspositionTable {
    table: Box<[TTEntry]>,
    size: usize,
    num_valid_entries: usize,
}

impl TranspositionTable {
    pub fn new(size_bytes: usize) -> Self {
        let values = vec![TTEntry::default(); size_bytes / std::mem::size_of::<TTEntry>()];
        let len = values.len();
        Self {
            table: values.into_boxed_slice(),
            size: len,
            num_valid_entries: 0,
        }
    }

    pub fn get(&self, pos: &Position) -> Option<TTEntry> {
        let hash = pos.board().get_hash();
        let index = self.to_entry_hash(hash);
        let mut entry = self.table[index];

        (entry.flag != EntryType::Invalid && entry.hash == hash).then(|| {
            entry.eval = entry.eval.add_ply(pos.ply().into());
            entry
        })
    }

    pub fn set(&mut self, pos: &Position, mut entry: TTEntry) -> bool {
        let index = self.to_entry_hash(entry.hash);
        let old_entry = self.table[index];

        let mut replace = false;

        replace |= old_entry.hash == entry.hash && entry.flag == EntryType::Exact;

        replace |= entry.depth >= old_entry.depth;

        if replace {
            if old_entry.flag == EntryType::Invalid {
                self.num_valid_entries += 1;
            }

            entry.eval = entry.eval.sub_ply(pos.ply().into());
            self.table[index] = entry;
        }

        replace
    }

    pub fn increment_age(&mut self) {
        self.table
            .iter_mut()
            .for_each(|e| e.depth = e.depth.saturating_sub(1))
    }

    pub fn hashfull(&self) -> usize {
        self.num_valid_entries * 1_000 / self.table.len()
    }

    // Knuth's method
    fn to_entry_hash(&self, original_hash: u64) -> usize {
        // let c: f64 = -1. + 5.0_f64.sqrt();

        // (self.size as f64 * (c * original_hash as f64).fract()).floor() as usize

        // original_hash as usize % self.size
        // ((original_hash as u32 as u64 * self.size as u64) >> u32::BITS) as usize
        // ((original_hash as u128 * 18446744073709551557) as usize) % self.size
        ((original_hash as u128 * self.size as u128) >> 64) as usize
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(1024)
    }
}

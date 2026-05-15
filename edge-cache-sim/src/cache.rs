//! Per-edge-node chunk stores (set membership + capacity).

use crate::types::ChunkId;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct NodeChunkStore {
    pub capacity: usize,
    pub chunks: HashSet<ChunkId>,
}

impl NodeChunkStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            chunks: HashSet::new(),
        }
    }

    pub fn contains(&self, c: ChunkId) -> bool {
        self.chunks.contains(&c)
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_full(&self) -> bool {
        self.chunks.len() >= self.capacity
    }

    pub fn insert(&mut self, c: ChunkId) -> bool {
        self.chunks.insert(c)
    }

    pub fn remove(&mut self, c: &ChunkId) -> bool {
        self.chunks.remove(c)
    }

    pub fn iter(&self) -> impl Iterator<Item = ChunkId> + '_ {
        self.chunks.iter().copied()
    }
}

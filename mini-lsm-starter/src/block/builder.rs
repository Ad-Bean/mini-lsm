use bytes::BufMut;

use crate::key::{KeySlice, KeyVec};

use super::Block;

/// Builds a block.
pub struct BlockBuilder {
    /// Offsets of each key-value entries.
    offsets: Vec<u16>,
    /// All serialized key-value pairs in the block.
    data: Vec<u8>,
    /// The expected block size.
    block_size: usize,
    // The first key in the block.
    first_key: KeyVec,
}

impl BlockBuilder {
    /// Creates a new block builder.
    pub fn new(block_size: usize) -> Self {
        Self {
            offsets: Vec::new(),
            data: Vec::new(),
            block_size,
            first_key: KeyVec::new(),
        }
    }

    /// Adds a key-value pair to the block. Returns false when the block is full.
    #[must_use]
    pub fn add(&mut self, key: KeySlice, value: &[u8]) -> bool {
        // data: [u8] | offsets: [u16] | num of elements: u16
        // println!("block_size: {}", self.block_size);
        // println!(
        //     "incoming block size: {}",
        //     self.data.len() + self.offsets.len() * 2 + 2 + key.len() + 2 + value.len() + 2 + 2
        // );
        if !self.is_empty() && /* can store one large key */
        self.data.len() + self.offsets.len() * 2 + 2 +  /* current encode */
        key.len() + 2 + value.len() + 2 + 2 /* key_len: 2b | key: keylen | value_len: 2b | value: val_len + offset */
        > self.block_size
        {
            return false;
        }

        if self.offsets.is_empty() {
            self.first_key = key.to_key_vec();
        }
        self.data.put_u16(key.len() as u16);
        self.data.put(key.into_inner());
        self.data.put_u16(value.len() as u16);
        self.data.put(value);

        self.offsets.push(self.data.len() as u16);
        true
    }

    /// Check if there is no key-value pair in the block.
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Finalize the block.
    pub fn build(self) -> Block {
        Block {
            offsets: self.offsets,
            data: self.data,
        }
    }
}

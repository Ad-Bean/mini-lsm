mod builder;
mod iterator;

pub use builder::BlockBuilder;
use bytes::{Buf, BufMut, Bytes};
pub use iterator::BlockIterator;

/// A block is the smallest unit of read and caching in LSM tree. It is a collection of sorted key-value pairs.
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u16>,
}

impl Block {
    /// Encode the internal data to the data layout illustrated in the tutorial
    /// Note: You may want to recheck if any of the expected field is missing from your output
    pub fn encode(&self) -> Bytes {
        let mut buf = self.data.clone();
        for offset in &self.offsets {
            buf.put_u16(*offset);
        }
        // num of elements
        buf.put_u16(self.offsets.len() as u16);
        buf.into()
    }

    /// Decode from the data layout, transform the input `data` to a single `Block`
    pub fn decode(data: &[u8]) -> Self {
        // the last u16 is the num of elements
        let num_of_elements = (&data[data.len() - 2..]).get_u16() as usize;
        let offsets = (&data[data.len() - 2 - num_of_elements * 2..data.len() - 2])
            .chunks(2)
            .map(|mut chunk| chunk.get_u16())
            .collect();
        let data = data[..data.len() - 2 - num_of_elements * 2].to_vec();
        Self { data, offsets }
    }
}

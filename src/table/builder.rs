use std::fmt::{Display, Formatter, Error};
use crate::table::sctable::{ScTableIndex, TABLE_MAGIC};
use crate::table::sctable::{TABLE_MIN_SIZE, TABLE_INDEX_SIZE};
use crate::encode::{encode_fixed32_ret, encode_fixed32};
use crc::crc32;

pub(crate) struct ScTableBuilder {
    indexes: Vec<ScTableIndex>,
    data: Vec<u8>
}

impl Default for ScTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScTableBuilder {
    pub(crate) fn new() -> Self {
        Self { indexes: Vec::new(), data: Vec::new() }
    }

    pub(crate) fn add_kv(&mut self, key: &[u8], value: &[u8]) {
        let key_off = self.data.len() as u32;
        let key_size = key.len() as u32;
        self.data.extend_from_slice(key);

        let value_off = self.data.len() as u32;
        let value_size = value.len() as u32;
        self.data.extend_from_slice(value);

        self.indexes.push(ScTableIndex::new(key_off, key_size, value_off, value_size));
    }

    pub(crate) fn build(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(self.size());
        ret.extend_from_slice(&encode_fixed32_ret((self.indexes.len() * TABLE_INDEX_SIZE) as u32));
        ret.extend_from_slice(&encode_fixed32_ret(self.data.len() as u32));
        for _ in 0..4 {
            ret.push(0)
        }
        ret.extend_from_slice(&encode_fixed32_ret(crc32::checksum_ieee(&self.data)));
        for index in self.indexes.iter() {
            index.serialize(&mut ret)
        }
        let index_checksum = crc32::checksum_ieee(&ret[16..(self.indexes.len() + 1) * 16]);
        encode_fixed32(&mut ret[8..12], index_checksum);
        ret.extend_from_slice(TABLE_MAGIC);
        ret
    }

    pub(crate) fn size(&self) -> usize {
        TABLE_MIN_SIZE + self.indexes.len() * TABLE_INDEX_SIZE + self.data.len()
    }
}

use crate::error::{Error, ErrorCode, ErrorStr};
use crate::encode::{decode_fixed32, encode_fixed32, encode_fixed32_ret};
use crc::crc32;

pub(crate) const TABLE_MAGIC: &'static [u8] = b"40490fd0";
pub(crate) const TABLE_MAGIC_SIZE: usize = TABLE_MAGIC.len();
pub(crate) const TABLE_HEAD_SIZE: usize = 16;
pub(crate) const TABLE_MIN_SIZE: usize = TABLE_MAGIC_SIZE + TABLE_HEAD_SIZE;

#[derive(Ord, Eq, PartialOrd, PartialEq, Hash, Copy, Clone)]
pub(crate) struct ScTableFile {
    origin_partition: u32,
    origin_level: u32,
    origin_number: u64
}

pub(crate) struct ScTableMeta {
    table_file: ScTableFile,

    key_lower_bound: String,
    key_upper_bound: String
}

pub(crate) const TABLE_INDEX_SIZE: usize = 16;

pub(crate) struct ScTableIndex {
    key_off: u32,
    key_len: u32,
    value_off: u32,
    value_len: u32
}

impl ScTableIndex {
    pub(crate) fn new(key_off: u32, key_len: u32, value_off: u32, value_len: u32) -> Self {
        Self { key_off, key_len, value_off, value_len }
    }

    pub(crate) fn serialize(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(&encode_fixed32_ret(self.key_off));
        dest.extend_from_slice(&encode_fixed32_ret(self.key_len));
        dest.extend_from_slice(&encode_fixed32_ret(self.value_off));
        dest.extend_from_slice(&encode_fixed32_ret(self.value_len));
    }

    pub(crate) fn deserialize(from: &[u8]) -> Self {
        debug_assert_eq!(from.len(), TABLE_INDEX_SIZE);
        Self {
            key_off: decode_fixed32(&from[0..4]),
            key_len: decode_fixed32(&from[4..8]),
            value_off: decode_fixed32(&from[8..12]),
            value_len: decode_fixed32(&from[12..16]),
        }
    }
}

pub(crate) struct ScTable {
    indexes: Vec<ScTableIndex>,
    data: Vec<u8>
}

impl ScTable {
    pub(crate) fn from_raw(raw: &[u8]) -> Result<ScTable, Error> {
        if raw.len() < TABLE_MIN_SIZE {
            return Err(Error::sc_table_corrupt("too small to be a table file".into()))
        }

        if &raw[raw.len()-TABLE_MAGIC_SIZE .. raw.len()] != TABLE_MAGIC {
            return Err(Error::sc_table_corrupt("incorrect table magic".into()))
        }

        let kv_index_size = decode_fixed32(&raw[0..4]) as usize;
        let data_size = decode_fixed32(&raw[4..8]) as usize;

        if kv_index_size % TABLE_INDEX_SIZE != 0 {
            return Err(Error::sc_table_corrupt("index size should be multiplication of 16".into()))
        }

        if (kv_index_size + data_size + TABLE_MIN_SIZE) != raw.len() {
            return Err(Error::sc_table_corrupt("incorrect table size".into()))
        }

        let kv_index_crc = decode_fixed32(&raw[8..12]);
        let data_crc = decode_fixed32(&raw[12..16]);

        let kv_index = &raw[TABLE_HEAD_SIZE..TABLE_HEAD_SIZE+kv_index_size];
        let data = &raw[TABLE_HEAD_SIZE+kv_index_size..TABLE_HEAD_SIZE+kv_index_size+data_size];

        if crc32::checksum_ieee(kv_index) != kv_index_crc {
            return Err(Error::sc_table_corrupt("incorrect kv_index crc".into()))
        }

        if crc32::checksum_ieee(data) != data_crc {
            return Err(Error::sc_table_corrupt("incorrect data crc".into()))
        }

        let mut indexes = Vec::new();
        for i in 0..kv_index_size / TABLE_INDEX_SIZE {
            let base = i * TABLE_INDEX_SIZE;
            let index = ScTableIndex::deserialize(&kv_index[base..base + TABLE_INDEX_SIZE]);
            if (index.key_off + index.key_len) as usize >= data.len()
                || (index.value_off + index.value_len) as usize >= data.len() {
                return Err(Error::sc_table_corrupt("incorrect key/value index data".into()))
            }
            indexes.push(index)
        }

        Ok(Self { indexes, data: data.to_vec() })
    }
}

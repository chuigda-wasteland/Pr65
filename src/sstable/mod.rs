use crate::error::{Error, ErrorCode, ErrorStr};
use crate::encode::{decode_fixed32, encode_fixed32};
use crc::crc32;

const TABLE_MAGIC: &'static [u8] = b"40490fd0";
const TABLE_MAGIC_SIZE: usize = TABLE_MAGIC.len();
const TABLE_HEAD_SIZE: usize = 16;

const TABLE_MIN_SIZE: usize = TABLE_MAGIC_SIZE + TABLE_HEAD_SIZE;

struct ScTableMeta {
    origin_partition: u32,
    origin_level: u32,
    origin_number: u64,
    key_lower_bound: String,
    key_upper_bound: String
}

struct ScTableIndex {
    key_off: u32,
    key_len: u32,
    value_off: u32,
    value_len: u32
}

struct ScTable {
    table_info: ScTableMeta,
    indexes: Vec<ScTableIndex>,
    data: Vec<u8>,
    filter: Option<Vec<u8>>
}

impl ScTable {
    pub(crate) fn from_raw(raw: &[u8]) -> Result<ScTable, Error> {
        if raw.len() < 24 {
            return Err(Error::sc_table_corrupt(ErrorStr::from("too small to be a table file")))
        }

        if &raw[raw.len()-8..raw.len()] != TABLE_MAGIC {
            return Err(Error::sc_table_corrupt(ErrorStr::from("incorrect table magic")))
        }

        let kv_index_size = decode_fixed32(&raw[0..4]) as usize;
        let data_size = decode_fixed32(&raw[4..8]) as usize;
        if (kv_index_size + data_size + TABLE_MIN_SIZE) != raw.len() {
            return Err(Error::sc_table_corrupt(ErrorStr::from("incorrect table size")))
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

        unimplemented!()
    }
}

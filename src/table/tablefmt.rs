use crate::encode::{encode_fixed32_ret, decode_fixed32};

pub(crate) const TABLE_MAGIC: &'static [u8] = b"40490fd0";
pub(crate) const TABLE_MAGIC_SIZE: usize = TABLE_MAGIC.len();
pub(crate) const TABLE_HEAD_SIZE: usize = 16;
pub(crate) const TABLE_MIN_SIZE: usize = TABLE_MAGIC_SIZE + TABLE_HEAD_SIZE;
pub(crate) const TABLE_INDEX_SIZE: usize = 16;

pub(crate) struct ScTableCatalogItem {
    pub(crate) key_off: u32,
    pub(crate) key_len: u32,
    pub(crate) value_off: u32,
    pub(crate) value_len: u32
}

impl ScTableCatalogItem {
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

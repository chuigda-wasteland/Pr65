use crc::crc32;

use crate::table::tablefmt::{TABLE_MAGIC, TABLE_MIN_SIZE, TABLE_CATALOG_ITEM_SIZE};
use crate::encode::{encode_fixed32_ret, encode_fixed32};
use crate::table::cache::ScTableCatalogItem;

pub(crate) struct ScTableBuilder {
    indexes: Vec<ScTableCatalogItem>,
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

    pub(crate) fn add_kv(&mut self, key_seq: u64, key: &[u8], value: &[u8]) {
        let key_off = self.data.len() as u32;
        let key_size = key.len() as u32;
        self.data.extend_from_slice(key);

        let value_off = self.data.len() as u32;
        let value_size = value.len() as u32;
        self.data.extend_from_slice(value);

        self.indexes.push(ScTableCatalogItem::new(key_seq, key_off, key_size, value_off, value_size));
    }

    pub(crate) fn build(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(self.size());
        ret.extend_from_slice(&encode_fixed32_ret((self.indexes.len() * TABLE_CATALOG_ITEM_SIZE) as u32));
        ret.extend_from_slice(&encode_fixed32_ret(self.data.len() as u32));
        for _ in 0..4 {
            ret.push(0)
        }
        ret.extend_from_slice(&encode_fixed32_ret(crc32::checksum_ieee(&self.data)));
        for index in self.indexes.iter() {
            index.serialize(&mut ret)
        }
        ret.extend_from_slice(&self.data);
        let index_checksum = crc32::checksum_ieee(&ret[16..16 + self.indexes.len() * TABLE_CATALOG_ITEM_SIZE]);
        encode_fixed32(&mut ret[8..12], index_checksum);
        ret.extend_from_slice(TABLE_MAGIC);
        ret
    }

    pub(crate) fn size(&self) -> usize {
        TABLE_MIN_SIZE + self.indexes.len() * TABLE_CATALOG_ITEM_SIZE + self.data.len()
    }
}

#[cfg(test)]
mod test {
    use crate::table::builder::ScTableBuilder;
    use crate::table::cache::{ScTableCache, TableCacheManager};

    #[test]
    fn test_builder_1() {
        let data = [
            (0x40490fd0fffffffeu64, "正当梨花开遍了天涯".as_bytes(), "Расцветали яблони и груши".as_bytes()),
            (0x40490fd0fffffffeu64, "河上飘着柔软的轻纱".as_bytes(), "Поплыли туманы над рекой".as_bytes()),
            (0x40490fd0fffffffeu64, "喀秋莎站在那俊俏的岸上".as_bytes(), "Выходила на берег Катюша".as_bytes()),
            (0x40490fd0fffffffeu64, "歌声好像明媚的春光".as_bytes(), "На высокий берег, на крутой".as_bytes()),
            (0x40490fd0fffffffeu64, "间奏".as_bytes(), "".as_bytes()),
            (0x40490fd0ffffffffu64, "喀秋莎站在那俊俏的岸上".as_bytes(), "Выходила на берег Катюша".as_bytes()),
            (0x40490fd0ffffffffu64, "歌声好像明媚的春光".as_bytes(), "На высокий берег, на крутой".as_bytes()),
            (0x40490fd0fffffffeu64, "尾声".as_bytes(), "".as_bytes()),
        ];

        let mut builder = ScTableBuilder::new();
        for &(seq, key, value) in data.iter() {
            builder.add_kv(seq, key, value);
        }
        let buffer = builder.build();

        let cache_manager = TableCacheManager::new(1);
        let quota = cache_manager.acquire_quota();
        let table = ScTableCache::from_raw(&buffer, quota).unwrap();
        assert_eq!(table.catalog_size(), data.len());
        for (i, &(seq, key, value)) in data.iter().enumerate() {
            let (seq1, key1, value1) = table.nth_item(i);
            assert_eq!(seq1, seq);
            assert_eq!(key1, key);
            assert_eq!(value1, value);
        }
    }
}

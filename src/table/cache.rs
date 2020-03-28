use std::sync::{Arc, Mutex};
use std_semaphore::Semaphore;

use lru::LruCache;
use crc::crc32;

use crate::table::sctable::ScTableFile;

use crate::table::tablefmt::{TABLE_MIN_SIZE, TABLE_MAGIC_SIZE, TABLE_MAGIC,
                             TABLE_CATALOG_ITEM_SIZE, TABLE_HEAD_SIZE};
use crate::encode::{encode_fixed32_ret, decode_fixed32};
use crate::error::Error;
use crate::Comparator;

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
        debug_assert_eq!(from.len(), TABLE_CATALOG_ITEM_SIZE);
        Self {
            key_off: decode_fixed32(&from[0..4]),
            key_len: decode_fixed32(&from[4..8]),
            value_off: decode_fixed32(&from[8..12]),
            value_len: decode_fixed32(&from[12..16]),
        }
    }
}


pub(crate) struct ScTableCache<'a> {
    catalog: Vec<ScTableCatalogItem>,
    data: Vec<u8>,
    quota: CacheQuota<'a>
}

impl<'a> ScTableCache<'a> {
    pub(crate) fn from_raw(raw: &[u8], quota: CacheQuota<'a>) -> Result<ScTableCache<'a>, Error> {
        if raw.len() < TABLE_MIN_SIZE {
            return Err(Error::sc_table_corrupt("too small to be a table file".into()))
        }

        if &raw[raw.len()-TABLE_MAGIC_SIZE .. raw.len()] != TABLE_MAGIC {
            return Err(Error::sc_table_corrupt("incorrect table magic".into()))
        }

        let kv_catalog_size = decode_fixed32(&raw[0..4]) as usize;
        let data_size = decode_fixed32(&raw[4..8]) as usize;

        if kv_catalog_size % TABLE_CATALOG_ITEM_SIZE != 0 {
            return Err(Error::sc_table_corrupt("catalog size should be multiplication of 16".into()))
        }

        if (kv_catalog_size + data_size + TABLE_MIN_SIZE) != raw.len() {
            return Err(Error::sc_table_corrupt("incorrect table size".into()))
        }

        let kv_catalog_crc = decode_fixed32(&raw[8..12]);
        let data_crc = decode_fixed32(&raw[12..16]);

        let kv_catalog = &raw[TABLE_HEAD_SIZE..TABLE_HEAD_SIZE+ kv_catalog_size];
        let data = &raw[TABLE_HEAD_SIZE+ kv_catalog_size..TABLE_HEAD_SIZE+ kv_catalog_size +data_size];

        if crc32::checksum_ieee(kv_catalog) != kv_catalog_crc {
            return Err(Error::sc_table_corrupt("incorrect kv_catalog crc".into()))
        }

        if crc32::checksum_ieee(data) != data_crc {
            return Err(Error::sc_table_corrupt("incorrect data crc".into()))
        }

        let mut catalog_item = Vec::new();
        for i in 0..kv_catalog_size / TABLE_CATALOG_ITEM_SIZE {
            let base = i * TABLE_CATALOG_ITEM_SIZE;
            let index =
                ScTableCatalogItem::deserialize(&kv_catalog[base..base + TABLE_CATALOG_ITEM_SIZE]);
            if (index.key_off + index.key_len) as usize >= data.len()
                || (index.value_off + index.value_len) as usize >= data.len() {
                return Err(Error::sc_table_corrupt("incorrect key/value catalog data".into()))
            }
            catalog_item.push(index)
        }

        Ok(Self { catalog: catalog_item, data: data.to_vec(), quota })
    }

    pub(crate) fn get<Comp: Comparator>(&self, key: &[u8]) -> Option<Vec<u8>> {
        if let Ok(idx) = self.catalog.binary_search_by(
            |catalog_item| Comp::compare(self.key(catalog_item), key)) {
            Some(self.value(&self.catalog[idx]).to_vec())
        } else {
            None
        }
    }

    fn key(&self, catalog_item: &ScTableCatalogItem) -> &[u8] {
        &self.data[catalog_item.key_off as usize .. (catalog_item.key_off + catalog_item.key_len) as usize]
    }

    fn value(&self, catalog_item: &ScTableCatalogItem) -> &[u8] {
        &self.data[catalog_item.value_off as usize .. (catalog_item.value_off + catalog_item.value_len) as usize]
    }
}


pub(crate) struct CacheQuota<'a> {
    cache_manager: &'a TableCacheManager<'a>
}

impl<'a> CacheQuota<'a> {
    fn new(cache_manager: &'a TableCacheManager<'a>) -> Self {
        Self { cache_manager }
    }
}

impl<'a> Drop for CacheQuota<'a> {
    fn drop(&mut self) {
        self.cache_manager.on_cache_released()
    }
}

pub(crate) struct TableCacheManager<'a> {
    lru: Mutex<LruCache<ScTableFile, Arc<ScTableCache<'a>>>>,
    sem: Semaphore
}

impl<'a> TableCacheManager<'a> {
    pub(crate) fn new(cache_count: usize) -> Self {
        TableCacheManager {
            lru: Mutex::new(LruCache::new(cache_count)),
            sem: Semaphore::new(cache_count as isize)
        }
    }

    pub(crate) fn acquire_quota(&'a self) -> CacheQuota<'a> {
        self.sem.acquire();
        CacheQuota::new(self)
    }

    pub(crate) fn add_cache(&'a self, table_file: ScTableFile, table_cache: ScTableCache<'a>) -> Arc<ScTableCache<'a>> {
        let ret = Arc::new(table_cache);
        self.lru.lock().unwrap().put(table_file, ret.clone());
        ret
    }

    pub(crate) fn get_cache(&'a self, table_file: ScTableFile) -> Option<Arc<ScTableCache<'a>>> {
        self.lru.lock().unwrap().get(&table_file).and_then(|arc| Some(arc.clone()))
    }

    fn on_cache_released(&self) {
        self.sem.release()
    }
}

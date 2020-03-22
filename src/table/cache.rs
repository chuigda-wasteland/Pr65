use lru::LruCache;
use crate::table::sctable::{ScTableFile, ScTable};

pub(crate) struct TableCacheManager {
    lru: LruCache<ScTableFile, ScTable>
}

impl TableCacheManager {
    pub(crate) fn new(cache_count: usize) -> Self {
        TableCacheManager {
            lru: LruCache::new(cache_count)
        }
    }
}
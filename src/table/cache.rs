use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ops::Deref;

use lru::LruCache;

use crate::table::sctable::{ScTableFile, ScTable};

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
    lru: Mutex<LruCache<ScTableFile, Arc<ScTable<'a>>>>,
    cache_count: usize,
    current_cache_count: AtomicUsize
}

impl<'a> TableCacheManager<'a> {
    pub(crate) fn new(cache_count: usize) -> Self {
        TableCacheManager {
            lru: Mutex::new(LruCache::new(cache_count)),
            cache_count,
            current_cache_count: AtomicUsize::new(0)
        }
    }

    pub(crate) fn allocate_quota(&'a self) -> CacheQuota<'a> {
        while self.current_cache_count.load(Ordering::SeqCst) >= self.cache_count {
        }
        self.current_cache_count.fetch_add(1, Ordering::SeqCst);
        CacheQuota::new(self)
    }

    pub(crate) fn add_cache(&'a self, table_file: ScTableFile, table_cache: ScTable<'a>) -> Arc<ScTable<'a>> {
        let ret = Arc::new(table_cache);
        self.lru.lock().unwrap().put(table_file, ret.clone());
        ret
    }

    pub(crate) fn get_cache(&'a self, table_file: ScTableFile) -> Option<Arc<ScTable<'a>>> {
        self.lru.lock().unwrap().get(&table_file).and_then(|arc| Some(arc.clone()))
    }

    fn on_cache_released(&self) {
        self.current_cache_count.fetch_sub(1, Ordering::SeqCst);
    }
}

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ops::Deref;

use lru::LruCache;

use crate::table::sctable::{ScTableFile, ScTable};

pub(crate) struct ScTableHandle<'a> {
    sc_table: ScTable,
    cache_manager: &'a TableCacheManager<'a>
}

impl<'a> ScTableHandle<'a> {
    fn new(sc_table: ScTable, cache_manager: &'a TableCacheManager<'a>) -> Self {
        Self { sc_table, cache_manager }
    }
}

impl<'a> Deref for ScTableHandle<'a> {
    type Target = ScTable;

    fn deref(&self) -> &Self::Target {
        &self.sc_table
    }
}

impl<'a> Drop for ScTableHandle<'a> {
    fn drop(&mut self) {
        self.cache_manager.on_cache_released()
    }
}

pub(crate) struct TableCacheManager<'a> {
    lru: Mutex<LruCache<ScTableFile, Arc<ScTableHandle<'a>>>>,
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

    pub(crate) fn add_cache(&'a self, table_file: ScTableFile, table_cache: ScTable) -> Arc<ScTableHandle<'a>> {
        while self.current_cache_count.load(Ordering::SeqCst) >= self.cache_count {}
        self.current_cache_count.fetch_add(1, Ordering::SeqCst);
        let ret = Arc::new(ScTableHandle::new(table_cache, self));
        self.lru.lock().unwrap().put(table_file, ret.clone());
        ret
    }

    pub(crate) fn get_cache(&'a self, table_file: ScTableFile) -> Option<Arc<ScTableHandle<'a>>> {
        self.lru.lock().unwrap().get(&table_file).and_then(|arc| Some(arc.clone()))
    }

    fn on_cache_released(&self) {
        self.current_cache_count.fetch_sub(1, Ordering::SeqCst);
    }
}

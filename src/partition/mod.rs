use std::collections::BTreeMap;
use crate::table::Table;
use crate::table::cache::TableCacheManager;
use std::sync::{Arc, Mutex, RwLock};
use crate::Comparator;
use std::marker::PhantomData;
use std::cmp::Ordering;

pub(crate) struct UserKey<Comp: Comparator> {
    key: Vec<u8>,
    phantom: PhantomData<Comp>
}

impl<Comp: Comparator> UserKey<Comp> {
    fn new(key: Vec<u8>) -> Self {
        Self { key, phantom: PhantomData }
    }
}

impl<Comp: Comparator> Ord for UserKey<Comp> {
    fn cmp(&self, other: &Self) -> Ordering {
        Comp::compare(&self.key, &other.key)
    }
}

impl<Comp: Comparator> PartialOrd for UserKey<Comp> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Comp: Comparator> PartialEq for UserKey<Comp> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<Comp: Comparator> Eq for UserKey<Comp> {}

type MemTable<Comp> = BTreeMap<UserKey<Comp>, Vec<u8>>;

struct Partition<'a, Comp: Comparator> {
    mem_table: RwLock<MemTable<Comp>>,
    imm_table: Mutex<Option<MemTable<Comp>>>,
    levels: Vec<Vec<Box<dyn Table<Comp>>>>,
    cache_manager: &'a Mutex<TableCacheManager>
}

impl<'a, Comp: Comparator> Partition<'a, Comp> {
    pub(crate) fn new(mem_table: MemTable<Comp>,
           imm_table: MemTable<Comp>,
           levels: Vec<Vec<Box<dyn Table<Comp>>>>,
           cache_manager: &'a Mutex<TableCacheManager>) -> Self {
        Self {
            mem_table: RwLock::new(mem_table),
            imm_table: Mutex::new(Some(imm_table)),
            levels,
            cache_manager
        }
    }
}

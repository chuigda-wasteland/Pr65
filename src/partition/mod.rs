use std::collections::BTreeMap;
use crate::table::Table;
use crate::table::cache::TableCacheManager;
use std::sync::{Arc, Mutex, RwLock};
use crate::Comparator;
use std::marker::PhantomData;
use std::cmp::Ordering;
use crate::io::IOManager;

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

pub(crate) struct Partition<'a, Comp: Comparator> {
    lower_bound: RwLock<Vec<u8>>,
    upper_bound: RwLock<Vec<u8>>,

    mem_table: RwLock<MemTable<Comp>>,
    imm_table: Mutex<Option<MemTable<Comp>>>,
    levels: Vec<Vec<Box<dyn Table<Comp>>>>,
    cache_manager: &'a TableCacheManager<'a>,
    io_manager: &'a IOManager
}

impl<'a, Comp: Comparator> Partition<'a, Comp> {
    pub(crate) fn new(lower_bound: Vec<u8>,
                      upper_bound: Vec<u8>,
                      mem_table: MemTable<Comp>,
                      imm_table: MemTable<Comp>,
                      levels: Vec<Vec<Box<dyn Table<Comp>>>>,
                      cache_manager: &'a TableCacheManager<'a>,
                      io_manager: &'a IOManager) -> Self {
        Self {
            lower_bound: RwLock::new(lower_bound),
            upper_bound: RwLock::new(upper_bound),
            mem_table: RwLock::new(mem_table),
            imm_table: Mutex::new(Some(imm_table)),
            levels,
            cache_manager,
            io_manager
        }
    }

    pub(crate) fn explode(self) -> (Partition<'a, Comp>, Partition<'a, Comp>) {
        debug_assert!(self.imm_table.lock().unwrap().is_none());
        unimplemented!()
    }
}

impl<'a, Comp: Comparator> PartialOrd for Partition<'a, Comp> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if Comp::compare(&self.upper_bound.read().unwrap(),
                         &other.lower_bound.read().unwrap()) == Ordering::Less {
            return Some(Ordering::Less)
        } else if Comp::compare(&other.upper_bound.read().unwrap(),
                                &self.lower_bound.read().unwrap()) == Ordering::Less {
            return Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl<'a, Comp: Comparator> Ord for Partition<'a, Comp> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<'a, Comp: Comparator> PartialEq for Partition<'a, Comp> {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            debug_assert_ne!(self as *const Partition<'a, Comp> as *const (),
                             other as *const Partition<'a, Comp> as *const ());
        }
        debug_assert_ne!(Comp::compare(&self.lower_bound.read().unwrap(),
                                       &other.lower_bound.read().unwrap()), Ordering::Equal);
        debug_assert_ne!(Comp::compare(&self.upper_bound.read().unwrap(),
                                       &other.upper_bound.read().unwrap()), Ordering::Equal);
        false
    }
}

impl<'a, Comp: Comparator> Eq for Partition<'a, Comp> {}

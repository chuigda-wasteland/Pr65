use std::collections::BTreeMap;
use crate::table::Table;
use crate::table::cache::TableCacheManager;
use std::sync::{Arc, Mutex, RwLock};
use crate::Comparator;
use std::marker::PhantomData;
use std::cmp::Ordering;
use crate::io::IOManager;
use crate::error::Error;

pub(crate) enum UserKey<Comp: Comparator> {
    Owned(Vec<u8>, PhantomData<Comp>),
    Borrow(*const [u8])
}

impl<Comp: Comparator> UserKey<Comp> {
    fn new_owned(vec: Vec<u8>) -> Self {
        UserKey::Owned(vec, PhantomData)
    }

    fn new_borrow(slice: &[u8]) -> Self {
        UserKey::Borrow(slice as *const [u8])
    }

    fn key(&self) -> &[u8]{
        match self {
            UserKey::Owned(k, _) => k.as_slice(),
            &UserKey::Borrow(b) => unsafe { b.as_ref().unwrap() }
        }
    }
}

impl<Comp: Comparator> Ord for UserKey<Comp> {
    fn cmp(&self, other: &Self) -> Ordering {
        Comp::compare(&self.key(), &other.key())
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

type Level<Comp> = Vec<Box<dyn Table<Comp>>>;

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
    fn level_get(level: &Level<Comp>, key: &[u8],
                 cache_manager: &'a TableCacheManager<'a>,
                 io_manager: &'a IOManager) -> Result<Option<Vec<u8>>, Error> {
        if let Ok(idx) = level.binary_search_by(|table| table.cmp_key(key)) {
            level[idx].get(key, cache_manager, io_manager).and_then(
                |slice| Ok(Some(slice.to_vec()))
            )
        } else {
            Ok(None)
        }
    }

    pub(crate) fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        if Comp::compare(key, &self.lower_bound.read().unwrap()) == Ordering::Less {
            return Ok(None)
        } else if Comp::compare(key, &self.upper_bound.read().unwrap()) == Ordering::Greater {
            return Ok(None)
        }

        let user_key = UserKey::new_borrow(key);
        if let Some(v) = self.mem_table.read().unwrap().get(&user_key) {
            return Ok(Some(v.clone()))
        }
        if let Some(v) = self.imm_table.lock().unwrap().as_ref().and_then(
            |imm_table| imm_table.get(&user_key).and_then(|v| Some(v.clone()))) {
            return Ok(Some(v))
        }

        Ok(None)
    }
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

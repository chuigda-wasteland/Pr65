use std::collections::BTreeMap;
use std::sync::{Mutex, atomic::AtomicU64, Condvar, Arc};
use std::marker::PhantomData;
use std::cmp::Ordering;
use std::ptr::NonNull;

use crate::{Comparator, Options, DefaultComparator};
use crate::table::tablefmt::{TABLE_CATALOG_ITEM_SIZE, TABLE_MIN_SIZE};
use crate::table::builder::ScTableBuilder;
use crate::table::cache::TableCacheManager;
use crate::io::IOManager;
use crate::error::Error;
use crate::partition::level::Level;
use crate::table::sctable::{ScTable, ScTableFile};

mod level;

pub(crate) enum UserKey<Comp: Comparator> {
    Owned(Vec<u8>, PhantomData<Comp>),
    Borrow(NonNull<[u8]>)
}

impl<Comp: Comparator> Clone for UserKey<Comp> {
    fn clone(&self) -> Self {
        match self {
            UserKey::Owned(data, _) => UserKey::Owned(data.clone(), PhantomData),
            UserKey::Borrow(ptr) => UserKey::Borrow(ptr.clone())
        }
    }
}

impl<Comp: Comparator> UserKey<Comp> {
    pub(crate) fn new_owned(vec: Vec<u8>) -> Self {
        UserKey::Owned(vec, PhantomData)
    }

    pub(crate) fn new_borrow(slice: &[u8]) -> Self {
        UserKey::Borrow(unsafe { NonNull::new_unchecked(slice as *const [u8] as _) })
    }

    fn key(&self) -> &[u8] {
        match self {
            UserKey::Owned(k, _) => k.as_slice(),
            UserKey::Borrow(b) => unsafe { b.as_ref() }
        }
    }

    fn is_owned(&self) -> bool {
        if let UserKey::Owned(_, _) = self {
            true
        } else {
            false
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

type DefaultUserKey = UserKey<DefaultComparator>;

pub(crate) struct InternalKey<Comp: Comparator> {
    seq: u64,
    pub(crate) user_key: UserKey<Comp>
}

impl<Comp: Comparator> InternalKey<Comp> {
    pub(crate) fn new(seq: u64, user_key: UserKey<Comp>) -> Self {
        Self { seq, user_key }
    }
}

impl<Comp: Comparator> Ord for InternalKey<Comp> {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord =  self.seq.cmp(&other.seq);
        if ord == Ordering::Equal {
            self.user_key.cmp(&other.user_key)
        } else {
            ord
        }
    }
}

impl<Comp: Comparator> PartialOrd for InternalKey<Comp> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Comp: Comparator> PartialEq for InternalKey<Comp> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<Comp: Comparator> Eq for InternalKey<Comp> {}

type MemTable<Comp> = BTreeMap<InternalKey<Comp>, Vec<u8>>;

pub(crate) struct Partition<'a, Comp: 'static + Comparator> {
    data: Mutex<PartitionData<'a, Comp>>,
    condvar: Condvar,

    partition_id: u32,

    seq: &'a AtomicU64,
    cache_manager: &'a TableCacheManager,
    io_manager: &'a IOManager,
    options: &'a Options
}

impl<'a, Comp: 'static + Comparator> Partition<'a, Comp> {
    fn new(options: &'a Options,
           partition_id: u32,
           seq: &'a AtomicU64,
           cache_manager: &'a TableCacheManager,
           io_manager: &'a IOManager) -> Self {
        Self {
            data: Mutex::new(PartitionData::new(options)),
            condvar: Condvar::new(),
            partition_id,
            seq,
            cache_manager,
            io_manager,
            options
        }
    }
}

impl<'a, Comp: Comparator> PartialOrd for Partition<'a, Comp> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let g1 = self.data.lock().unwrap();
        let g2 = other.data.lock().unwrap();
        let (self_lower, self_upper) = g1.bounds();
        let (other_lower, other_upper) = g2.bounds();

        if self_upper.unwrap().cmp(&other_lower.unwrap()) == Ordering::Less {
            return Some(Ordering::Less)
        } else if self_lower.unwrap().cmp(&other_upper.unwrap()) == Ordering::Greater {
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
        debug_assert!(Self::debug_never_eq_sanity_check(self, other));
        false
    }
}

impl<'a, Comp: Comparator> Partition<'a, Comp> {
    fn debug_never_eq_sanity_check(&self, other: &Self) -> bool {
        if self as *const Self == other as *const Self {
            return false;
        }

        let g1 = self.data.lock().unwrap();
        let g2 = other.data.lock().unwrap();
        let (self_lower, self_upper) = g1.bounds();
        let (other_lower, other_upper) = g2.bounds();
        if self_lower.is_some() && self_upper.is_some()
           && other_lower.is_some() && other_upper.is_some()
           && (self_lower.unwrap().cmp(other_lower.unwrap()) == Ordering::Equal
               || self_upper.unwrap().cmp(other_upper.unwrap()) == Ordering::Equal) {
            return false;
        }

        true
    }
}

impl<'a, Comp: Comparator> Eq for Partition<'a, Comp> {}

fn kv_pair_size<Comp>(key: &InternalKey<Comp>, value: &[u8]) -> usize
    where Comp: Comparator {
    key.user_key.key().len() + value.len() + TABLE_CATALOG_ITEM_SIZE
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
struct ArcPartition<'a, Comp: 'static + Comparator>(Arc<Partition<'a, Comp>>);

impl<'a, Comp: 'static + Comparator> ArcPartition<'a, Comp> {
    fn write(&self, key: InternalKey<Comp>, value: Vec<u8>) -> Result<(), Error> {
        let partition = &self.0;
        let mut data = partition.data.lock().unwrap();
        data.background_error()?;
        loop {
            if data.memtable_size() + kv_pair_size(&key, &value) <= partition.options.table_size {
                break;
            } else if data.has_imm() {
                data = partition.condvar.wait(data).unwrap();
            } else {
                data.convert_mem_to_imm();
                // TODO run self.do_compaction() at background
                break;
            }
        }
        data.memtable_put(key, value);
        Ok(())
    }

    fn compact_memtable(&self) {
        let partition = &self.0;
        let buffer;
        let file_number;
        let imm_bounds;
        {
            let data = partition.data.lock().unwrap();
            if data.background_error().is_err() {
                return;
            }
            imm_bounds = data.imm_bounds();
            let mut builder = ScTableBuilder::new();
            for (k, v) in data.imm_table.unwrap().iter() {
                builder.add_kv(k.seq, k.user_key.key(), &v);
            }
            buffer = builder.build();
            if data.levels.len() == 0 {
                data.levels.push(Level::new());
            }
            file_number = data.levels[0].level_next_file_id();
        }
        let table_file = ScTableFile::new(partition.partition_id, 0, file_number);
        if let Err(e) = partition.io_manager.acquire_quota().write_file(table_file.file_name(), &buffer) {
            partition.data.lock().unwrap().record_background_error(e);
            return;
        }
        let (imm_lower, imm_upper) = imm_bounds;
        let table = ScTable::new(table_file, imm_lower, imm_upper);
        {
            let data = partition.data.lock().unwrap();
            data.levels[0].add_file(table);
            // TODO flush metadata onto disk
            let _ = data.imm_table.take();
        }
        partition.condvar.notify_one();
        // TODO check if it is required to schedule another compaction from level 1 to level 2
    }
}

pub(crate) struct PartitionData<'a, Comp: 'static + Comparator> {
    mem_table: MemTable<Comp>,
    mem_table_data_size: usize,

    imm_table: Option<MemTable<Comp>>,
    levels: Vec<Level<Comp>>,

    lower_bound: Option<UserKey<Comp>>,
    upper_bound: Option<UserKey<Comp>>,

    background_error: Option<Error>,

    options: &'a Options
}

impl<'a, Comp: 'static + Comparator> PartitionData<'a, Comp> {
    fn new(options: &'a Options) -> Self {
        Self {
            mem_table: MemTable::new(),
            mem_table_data_size: 0,
            imm_table: None,
            levels: Vec::new(),
            lower_bound: None,
            upper_bound: None,
            background_error: None,
            options
        }
    }

    fn background_error(&self) -> Result<(), Error> {
        if let Some(e) = &self.background_error {
            Err(e.clone())
        } else {
            Ok(())
        }
    }

    fn record_background_error(&mut self, error: Error) {
        self.background_error.replace(error);
    }

    fn has_imm(&self) -> bool {
        self.imm_table.is_some()
    }

    fn imm_bounds(&self) -> (UserKey<Comp>, UserKey<Comp>) {
        let imm_table = self.imm_table.as_ref().unwrap();
        let (lower_bound, _) = imm_table.first_key_value().unwrap();
        let (upper_bound, _) = imm_table.last_key_value().unwrap();
        (lower_bound.user_key.clone(), upper_bound.user_key.clone())
    }

    fn memtable_put(&mut self, key: InternalKey<Comp>, value: Vec<u8>) {
        debug_assert!(self.memtable_size() + kv_pair_size(&key, &value) <= self.options.table_size);
        if self.lower_bound.is_none() && self.upper_bound.is_none() {
            self.set_lower_bound(key.user_key.clone());
            self.set_upper_bound(key.user_key.clone());
        } else if &key.user_key < self.lower_bound.as_ref().unwrap() {
            self.set_lower_bound(key.user_key.clone());
        } else if &key.user_key > self.upper_bound.as_ref().unwrap() {
            self.set_upper_bound(key.user_key.clone());
        }
        self.mem_table.insert(key, value);
    }

    fn convert_mem_to_imm(&mut self) {
        let new_imm = std::mem::replace(&mut self.mem_table, MemTable::new());
        self.imm_table.replace(new_imm);
    }

    fn memtable_size(&self) -> usize {
        self.mem_table_data_size + self.mem_table.len() * TABLE_CATALOG_ITEM_SIZE + TABLE_MIN_SIZE
    }

    fn bounds(&self) -> (Option<&UserKey<Comp>>, Option<&UserKey<Comp>>) {
        (self.lower_bound.as_ref(), self.upper_bound.as_ref())
    }

    fn set_lower_bound(&mut self, lower_bound: UserKey<Comp>) {
        debug_assert!(lower_bound.is_owned());
        self.lower_bound.replace(lower_bound);
    }

    fn set_upper_bound(&mut self, upper_bound: UserKey<Comp>) {
        debug_assert!(upper_bound.is_owned());
        self.upper_bound.replace(upper_bound);
    }

    fn debug_bounds_sanity_check(&self) -> bool {
        self.lower_bound.is_some() == self.upper_bound.is_some()
    }
}

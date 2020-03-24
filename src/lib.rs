#![feature(fn_traits)]
#![feature(with_options)]

use std::cmp::Ordering;
use std::marker::PhantomData;
use std::collections::VecDeque;

mod encode;
mod error;
mod table;
mod partition;
mod io;

pub trait Comparator {
    fn compare(lhs: &[u8], rhs: &[u8]) -> Ordering;
}

pub struct DefaultComparator();

impl Comparator for DefaultComparator {
    fn compare(lhs: &[u8], rhs: &[u8]) -> Ordering {
        lhs.cmp(rhs)
    }
}

pub struct Options {
    pub db_name: String,
    pub cache_count: usize,
    pub level0_size: usize,
    pub size_factor: usize,
    pub max_open_files: usize,
    pub table_size: usize,
    pub key_size_max: usize,
    pub value_size_max: usize
}

impl Options {
    pub fn new(db_name: impl ToString,
           cache_count: usize,
           level0_size: usize,
           size_factor: usize,
           max_open_files: usize,
           table_size: usize,
           key_size_max: usize,
           value_size_max: usize) -> Self {
        Self {
            db_name: db_name.to_string(),
            cache_count,
            level0_size,
            size_factor,
            max_open_files,
            table_size,
            key_size_max,
            value_size_max
        }
    }
}

use partition::Partition;
use std::sync::Mutex;
use crate::io::IOManager;
use crate::table::cache::TableCacheManager;

pub struct ScottDB<'a, Comp: Comparator> {
    phantom: PhantomData<Comp>,

    options: Options,
    partitions: VecDeque<Partition<'a, Comp>>,
    cache_manager: TableCacheManager<'a>,
    io_manager: IOManager
}

impl<'a, Comp: Comparator> ScottDB<'a, Comp> {
    pub fn new(options: Options) -> Self {
        let cache_count = options.cache_count;
        let max_open_files = options.max_open_files;
        Self {
            phantom: PhantomData,
            options,
            partitions: VecDeque::new(),
            cache_manager: TableCacheManager::new(cache_count),
            io_manager: IOManager::new(max_open_files)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

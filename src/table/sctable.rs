use std::cmp::Ordering;
use std::marker::PhantomData;

use crate::error::Error;
use crate::table::Table;
use crate::table::cache::{TableCacheManager, ScTableCache};
use crate::Comparator;
use crate::io::IOManager;

#[derive(Ord, Eq, PartialOrd, PartialEq, Hash, Copy, Clone)]
pub(crate) struct ScTableFile {
    origin_partition: u32,
    origin_level: u32,
    origin_number: u64
}

impl ScTableFile {
    fn file_name(&self) -> String {
        format!("{}_{}_{}.sst", self.origin_partition, self.origin_level, self.origin_number)
    }
}

pub(crate) struct ScTable<Comp> {
    table_file: ScTableFile,

    key_lower_bound: Vec<u8>,
    key_upper_bound: Vec<u8>,

    phantom: PhantomData<Comp>
}

impl<Comp: Comparator> Table<Comp> for ScTable<Comp> {
    fn get<'a>(&self,
               key: &[u8],
               cache_manager: &'a TableCacheManager<'a>,
               io_manager: &'a IOManager) -> Result<Option<Vec<u8>>, Error> {
        if Comp::compare(key, self.lower_bound()) == Ordering::Less {
            return Ok(None)
        } else if Comp::compare(key, self.upper_bound()) == Ordering::Greater {
            return Ok(None)
        }

        if let Some(cache) = cache_manager.get_cache(self.table_file) {
            Ok(cache.get::<Comp>(key))
        } else {
            let cache_quota = cache_manager.require_quota();
            let cache =
                ScTableCache::from_raw(
                    &io_manager.require_quota()
                                    .read_file(self.table_file.file_name())?, cache_quota)?;
            let cache = cache_manager.add_cache(self.table_file, cache);
            Ok(cache.get::<Comp>(key))
        }
    }

    fn lower_bound(&self) -> &[u8] {
        &self.key_lower_bound
    }

    fn upper_bound(&self) -> &[u8] {
        &self.key_upper_bound
    }

    fn is_lazy(&self) -> bool {
        false
    }
}

use std::cmp::Ordering;
use std::marker::PhantomData;

use crate::error::Error;
use crate::table::Table;
use crate::table::cache::{TableCacheManager, ScTableCache};
use crate::Comparator;
use crate::io::IOManager;
use crate::partition::{InternalKey, UserKey};

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

pub(crate) struct ScTable<Comp: Comparator> {
    table_file: ScTableFile,

    key_lower_bound: UserKey<Comp>,
    key_upper_bound: UserKey<Comp>
}

impl<Comp: Comparator> Table<Comp> for ScTable<Comp> {
    fn get<'a>(&self,
               key: &InternalKey<Comp>,
               cache_manager: &'a TableCacheManager,
               io_manager: &'a IOManager) -> Result<Option<Vec<u8>>, Error> {
        if key.user_key.cmp(self.lower_bound()) == Ordering::Less {
            return Ok(None)
        } else if key.user_key.cmp(self.upper_bound()) == Ordering::Greater {
            return Ok(None)
        }

        if let Some(cache) = cache_manager.get_cache(self.table_file) {
            Ok(cache.get::<Comp>(key))
        } else {
            let cache_quota = cache_manager.acquire_quota();
            let cache =
                ScTableCache::from_raw(
                    &io_manager.acquire_quota()
                                    .read_file(self.table_file.file_name())?, cache_quota)?;
            let cache = cache_manager.add_cache(self.table_file, cache);
            Ok(cache.get::<Comp>(key))
        }
    }

    fn lower_bound(&self) -> &UserKey<Comp> {
        &self.key_lower_bound
    }

    fn upper_bound(&self) -> &UserKey<Comp> {
        &self.key_upper_bound
    }

    fn is_lazy(&self) -> bool {
        false
    }
}

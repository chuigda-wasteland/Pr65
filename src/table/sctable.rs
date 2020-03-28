use std::cmp::Ordering;

use crc::crc32;

use crate::error::{Error};
use crate::encode::{decode_fixed32, encode_fixed32, encode_fixed32_ret};
use crate::table::Table;
use crate::table::cache::{TableCacheManager, CacheQuota};
use crate::Comparator;
use crate::io::IOManager;

#[derive(Ord, Eq, PartialOrd, PartialEq, Hash, Copy, Clone)]
pub(crate) struct ScTableFile {
    origin_partition: u32,
    origin_level: u32,
    origin_number: u64
}

pub(crate) struct ScTable {
    table_file: ScTableFile,

    key_lower_bound: Vec<u8>,
    key_upper_bound: Vec<u8>
}

impl<Comp: Comparator> Table<Comp> for ScTable {
    fn get<'a>(&self,
               key: &[u8],
               cache_manager: &'a TableCacheManager,
               io_manager: &'a IOManager) -> Result<Option<Vec<u8>>, Error> {
        unimplemented!()
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

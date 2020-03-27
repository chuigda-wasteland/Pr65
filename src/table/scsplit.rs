use std::cmp::Ordering;

use crate::table::sctable::ScTableFile;
use crate::Comparator;
use crate::table::Table;
use crate::table::cache::TableCacheManager;
use crate::io::IOManager;
use crate::error;

pub(crate) struct ScSplit {
    file: ScTableFile,

    first_kv_index: u32,
    last_kv_index: u32,

    lower_bound: Vec<u8>,
    upper_bound: Vec<u8>
}

impl<Comp: Comparator> Table<Comp> for ScSplit {
    fn get<'a>(&self,
               key: &[u8],
               cache_manager: &'a TableCacheManager,
               io_manager: &'a IOManager) -> Result<Option<Vec<u8>>, error::Error> {
        unimplemented!()
    }

    fn lower_bound(&self) -> &[u8] {
        &self.lower_bound
    }

    fn upper_bound(&self) -> &[u8] {
        &self.upper_bound
    }

    fn is_lazy(&self) -> bool {
        false
    }
}

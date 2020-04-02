use crate::table::sctable::ScTableFile;
use crate::Comparator;
use crate::table::Table;
use crate::table::cache::TableCacheManager;
use crate::io::IOManager;
use crate::error;
use crate::partition::{InternalKey, UserKey};

pub(crate) struct ScSplit<Comp: Comparator> {
    file: ScTableFile,

    first_kv_index: u32,
    last_kv_index: u32,

    lower_bound: UserKey<Comp>,
    upper_bound: UserKey<Comp>
}

impl<Comp: Comparator> Table<Comp> for ScSplit<Comp> {
    fn get<'a>(&self,
               key: &InternalKey<Comp>,
               cache_manager: &'a TableCacheManager,
               io_manager: &'a IOManager) -> Result<Option<Vec<u8>>, error::Error> {
        unimplemented!()
    }

    fn lower_bound(&self) -> &UserKey<Comp> {
        &self.lower_bound
    }

    fn upper_bound(&self) -> &UserKey<Comp> {
        &self.upper_bound
    }

    fn is_lazy(&self) -> bool {
        false
    }
}

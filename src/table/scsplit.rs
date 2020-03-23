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
               cache_manager: &'a mut TableCacheManager,
               io_manager: &'a IOManager) -> Result<&'a [u8], error::Error> {
        unimplemented!()
    }

    fn is_lazy(&self) -> bool {
        false
    }
}

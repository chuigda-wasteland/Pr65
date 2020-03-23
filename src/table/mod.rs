mod builder;
pub(crate) mod cache;

mod sctable;
mod scsplit;

use crate::{Comparator, error};
use crate::io::IOManager;
use crate::table::cache::TableCacheManager;

pub(crate) trait Table<Comp: Comparator> {
    fn get<'a>(&self,
               key: &[u8],
               cache_manager: &'a mut TableCacheManager,
               io_manager: &'a IOManager) -> Result<&'a [u8], error::Error>;

    fn is_lazy(&self) -> bool;
}

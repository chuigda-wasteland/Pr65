mod builder;
pub(crate) mod cache;

mod sctable;
mod scsplit;

use crate::table::cache::TableCacheManager;
use crate::Comparator;

pub(crate) trait Table<Comp: Comparator> {
    fn get<'a>(&self, key: &[u8], cache_manager: &'a mut TableCacheManager) -> &'a [u8];

    fn is_lazy(&self) -> bool;
}

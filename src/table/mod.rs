mod builder;
pub(crate) mod cache;

mod sctable;
mod scsplit;

use crate::table::cache::TableCacheManager;

pub(crate) trait Table {
    fn get<'a>(&self, key: &[u8], cache_manager: &'a mut TableCacheManager) -> &'a [u8];

    fn is_lazy(&self) -> bool;
}

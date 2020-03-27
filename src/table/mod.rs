mod builder;
pub(crate) mod cache;

mod sctable;
mod scsplit;

use std::cmp::Ordering;

use crate::{Comparator, error};
use crate::io::IOManager;
use crate::table::cache::TableCacheManager;

pub(crate) trait Table<Comp: Comparator> {
    fn get<'a>(&self,
               key: &[u8],
               cache_manager: &'a TableCacheManager,
               io_manager: &'a IOManager) -> Result<Option<Vec<u8>>, error::Error>;

    fn cmp_key(&self, key: &[u8]) -> Ordering {
        if Comp::compare(key, &self.lower_bound()) == Ordering::Less {
            Ordering::Less
        } else if Comp::compare(key, &self.upper_bound()) == Ordering::Greater {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    fn lower_bound(&self) -> &[u8];

    fn upper_bound(&self) -> &[u8];

    fn is_lazy(&self) -> bool;
}

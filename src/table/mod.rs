mod builder;
pub(crate) mod cache;
pub(crate) mod tablefmt;
mod sctable;
mod scsplit;

use std::cmp::Ordering;

use crate::{Comparator, error};
use crate::io::IOManager;
use crate::table::cache::TableCacheManager;
use crate::partition::{InternalKey, UserKey};

pub(crate) trait Table<Comp: Comparator> {
    fn get<'a>(&self,
               key: &InternalKey<Comp>,
               cache_manager: &'a TableCacheManager<'a>,
               io_manager: &'a IOManager) -> Result<Option<Vec<u8>>, error::Error>;

    fn cmp_key(&self, key: &UserKey<Comp>) -> Ordering {
        if key.cmp(self.lower_bound()) == Ordering::Less {
            Ordering::Less
        } else if key.cmp(self.upper_bound()) == Ordering::Greater {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    fn lower_bound(&self) -> &UserKey<Comp>;

    fn upper_bound(&self) -> &UserKey<Comp>;

    fn is_lazy(&self) -> bool;
}

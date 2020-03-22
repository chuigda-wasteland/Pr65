use std::collections::BTreeMap;
use crate::table::Table;
use crate::table::cache::TableCacheManager;
use std::sync::{Arc, Mutex};

struct Partition {
    mem_table: BTreeMap<Vec<u8>, Vec<u8>>,
    imm_table: BTreeMap<Vec<u8>, Vec<u8>>,
    levels: Vec<Vec<Box<dyn Table>>>,
    cache_manager: Arc<Mutex<TableCacheManager>>
}

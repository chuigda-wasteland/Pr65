use std::collections::BTreeMap;
use crate::table::Table;
use crate::table::cache::TableCacheManager;
use std::sync::{Arc, Mutex, RwLock};

struct Partition {
    mem_table: RwLock<BTreeMap<Vec<u8>, Vec<u8>>>,
    imm_table: Mutex<Option<BTreeMap<Vec<u8>, Vec<u8>>>>,
    levels: Vec<Vec<Box<dyn Table>>>,
    cache_manager: Mutex<TableCacheManager>
}

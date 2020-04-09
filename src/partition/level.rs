use crate::table::Table;
use crate::Comparator;
use crate::table::sctable::ScTable;

pub struct Level<Comp: 'static + Comparator> {
    tables: Vec<Box<dyn Table<Comp>>>,
    file_ids: Vec<u64>
}

impl<Comp: 'static + Comparator> Level<Comp> {
    pub(crate) fn new() -> Self {
        Self {
            tables: Vec::new(),
            file_ids: Vec::new()
        }
    }

    pub(crate) fn add_file(&mut self, table_file: ScTable<Comp>) {
        self.tables.push(Box::new(table_file));
    }

    pub(crate) fn level_next_file_id(&mut self, level: usize) -> u64 {
        let level_idx = level - 1;
        let ret = self.file_ids[level_idx];
        self.file_ids[level_idx] += 1;
        ret
    }
}

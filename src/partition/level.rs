use crate::table::Table;
use crate::Comparator;
use crate::table::sctable::ScTable;

pub struct Level<Comp: 'static + Comparator> {
    tables: Vec<Box<dyn Table<Comp>>>,
    file_id: u64
}

impl<Comp: 'static + Comparator> Level<Comp> {
    pub(crate) fn new() -> Self {
        Self {
            tables: Vec::new(),
            file_id: 1
        }
    }

    pub(crate) fn add_file(&mut self, table_file: ScTable<Comp>) {
        self.tables.push(Box::new(table_file));
    }

    pub(crate) fn level_next_file_id(&mut self) -> u64 {
        let ret = self.file_id;
        self.file_id += 1;
        ret
    }
}

use crate::table::sctable::ScTableFile;

pub(crate) struct ScSplit {
    file: ScTableFile,

    first_kv_index: u32,
    last_kv_index: u32,

    lower_bound: String,
    upper_bound: String
}
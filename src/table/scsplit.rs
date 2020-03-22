pub(crate) struct ScSplit {
    origin_partition: u32,
    origin_level: u32,
    origin_number: u64,

    first_kv_index: u32,
    last_kv_index: u32,

    lower_bound: String,
    upper_bound: String
}
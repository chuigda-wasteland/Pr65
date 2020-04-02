pub(crate) const TABLE_MAGIC: &'static [u8] = b"40490fd0";
pub(crate) const TABLE_MAGIC_SIZE: usize = TABLE_MAGIC.len();
pub(crate) const TABLE_HEAD_SIZE: usize = 16;
pub(crate) const TABLE_MIN_SIZE: usize = TABLE_MAGIC_SIZE + TABLE_HEAD_SIZE;
pub(crate) const TABLE_CATALOG_ITEM_SIZE: usize = 24;

pub(crate) const TABLE_MAX_SIZE: usize = 0x7FFFFFFF;
pub(crate) const TABLE_DELETION_BITMASK: u32 = 0x80000000;

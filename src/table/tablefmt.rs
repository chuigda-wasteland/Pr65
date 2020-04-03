//! Table format
//! ```raw
//! +-HEADER-------------------------------------+
//! | 4byte catalog size                         |
//! | 4byte data size                            |
//! | 4byte catalog crc                          |
//! | 4byte data crc                             |
//! +-CATALOG---+---------------+----------------+
//! | 8byte seq | 4byte key_off | 4byte key_size |
//! | 8byte seq | 4byte key_off | 4byte key_size |
//! | ...       | ...           | ...            |
//! +-DATA------+---------------+----------------+
//! | data_size binary data                      |
//! |                                            |
//! +-TAIL---------------------------------------+
//! | 8byte TABLE_MAGIC                          |
//! +--------------------------------------------+
//! ```

pub const TABLE_HEAD_SIZE: usize = 16;
pub const TABLE_MIN_SIZE: usize = TABLE_MAGIC_SIZE + TABLE_HEAD_SIZE;
pub const TABLE_CATALOG_ITEM_SIZE: usize = 24;

pub const TABLE_MAX_SIZE: usize = 0x7FFFFFFF;
pub const TABLE_DELETION_BITMASK: u32 = 0x80000000;

pub const TABLE_MAGIC: &'static [u8] = b"40490fd0";
pub const TABLE_MAGIC_SIZE: usize = TABLE_MAGIC.len();

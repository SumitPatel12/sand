#[derive(Debug)]
pub enum TextEncoding {
    UTF8,
    UTF16le,
    UTF16be,
}

// 100 byte DB header for the SQLite file.
#[derive(Debug)]
pub struct DBHeader {
    // Always: 53 51 4c 69 74 65 20 66 6f 72 6d 61 74 20 33 00 which is "SQLite format 3"
    pub header_string: [u8; 16],
    // Page size in bytes. Power of 2 between 512 and 32768 or 1, 1 representing 65536.
    pub page_size: u32,
    pub write_version: u8,
    pub read_version: u8,
    pub reserved_space_size: u8,
    pub max_payload_fraction: u8,  // Always 64
    pub min_payload_fraction: u8,  // Always 32
    pub leaf_payload_fraction: u8, // Always 32
    pub file_change_counter: u32,
    pub db_size_in_pages: u32,
    pub freelist_trunk_page: u32,
    pub total_freelist_pages: u32,
    pub schema_cookie: u32,
    pub schema_format_number: u32,
    pub default_page_cache_size: u32,
    // TODO: Come up with a better name.
    // Has a value if atuo-vacuum or incremental vaccum is on, otherwise it is 0.
    pub largest_btree_root_page: u32,
    pub text_encoding: TextEncoding,
    pub user_version: u32,
    pub incremental_vacuum_mode: u32,
    pub application_id: u32,
    pub reserved_space: [u8; 20],
    pub version_valid_for: u32,
    pub sqlite_version: u32,
}

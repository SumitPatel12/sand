/*
* 100 byte DB header for the SQLite file.
* The first page of the DB file is the file header. It gives the metadat for the database.
* The header file contains:
* 1.  Header string: Always "SQLite format 3".
* 2.  DB page size in bytes.
* 3.  File format write version. 1 is for legacy, 2 is for WAL.
* 4.  File format read version. 1 is for legacy, 2 is for WAL.
* 5.  Bytes of unused eserved space at the end of the page.
* 6.  Max embedded payload fraction. Is always 64.
* 7.  Min embedded payload fraction. Is always 32.
* 8.  Leaf payload fraction. Is always 32.
* 9.  File change counter.
* 10. The DB size in pages.
* 11. Page number of the first freelist trunk page.
* 12. Total number of freelist trunk pages.
* 13. Schema cookie.
* 14. Schema format number. 1,2,3, adn 4 are valid.
* 15. Default page cache size.
* 16. Page number of the largest root b-tree page when in auto-vaccum or incremental-vaccum mode. 0
*     for all other cases.
* 17. DB Text Encoding. 1 for UTF-8, 2 for UTF16le, and 3 for UTF-16be.
* 18. User Version.
* 19. Byte indicating incremental vaccum mode.
* 20. Application Id, which application is using/created it if any.
* 21. Reserved space for expansion. Will be all 0s.
* 22. Version valid for number.
* 23. SQLite version number.
*/
#[derive(Debug)]
pub struct DBHeader {
    pub header_string: [u8; 16],
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
    pub largest_btree_root_page: u32,
    pub text_encoding: TextEncoding,
    pub user_version: u32,
    pub incremental_vacuum_mode: u32,
    pub application_id: u32,
    pub reserved_space: [u8; 20],
    pub version_valid_for: u32,
    pub sqlite_version: u32,
}

#[derive(Debug)]
pub enum TextEncoding {
    UTF8,
    UTF16le,
    UTF16be,
}

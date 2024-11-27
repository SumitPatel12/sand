use super::{
    dbfileheader::{DBHeader, TextEncoding},
    errors::DBErrors,
    page::Page,
};
use std::convert::TryInto;

// File header parser
pub const HEADER_SIZE_BYTES: usize = 100;
const HEADER_STRING: &'static [u8; 16] = b"SQLite format 3\0";
const HEADER_PAGE_SIZE_OFFSET: usize = 16;
const MAX_PAGE_SIZE: u32 = 65536;
// The static offsets taken from the SQLite documentation
const HEADER_OFFSETS: [usize; 23] = [
    0, 16, 18, 19, 20, 21, 22, 23, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60, 64, 68, 72, 92, 96,
];

pub fn parse_file_header(buffer: &[u8]) -> Result<DBHeader, DBErrors> {
    let mut header_string = String::new();
    if !buffer.starts_with(HEADER_STRING) {
        header_string = String::from_utf8_lossy(&buffer[..HEADER_STRING.len()]).to_string();
        raise_error(header_string);
    }

    let mut offset = 16; // Start after HEADER_STRING

    let page_size = match u16::from_be_bytes(buffer[offset..offset + 2].try_into().unwrap()) {
        1 => MAX_PAGE_SIZE,
        page_size_bytes => page_size_bytes as u32,
    };
    if !page_size.is_power_of_two() {
        raise_error(format!("Page size is not power of 2: {}", page_size));
    }
    offset += 2;

    // Helper closure to extract values
    let get_u8 = |buf: &[u8], offset: &mut usize| -> u8 {
        let val = buf[*offset];
        *offset += 1;
        val
    };

    let get_u32 = |buf: &[u8], offset: &mut usize| -> u32 {
        let val = u32::from_be_bytes(buf[*offset..*offset + 4].try_into().unwrap());
        *offset += 4;
        val
    };

    // Extract remaining fields
    let write_version = get_u8(buffer, &mut offset);
    let read_version = get_u8(buffer, &mut offset);
    let reserved_space_size = get_u8(buffer, &mut offset);
    offset += 3; // Skip padding

    let file_change_counter = get_u32(buffer, &mut offset);
    let database_size_pages = get_u32(buffer, &mut offset);
    let first_freelist_trunk_page = get_u32(buffer, &mut offset);
    let number_of_freelist_pages = get_u32(buffer, &mut offset);
    let schema_cookie = get_u32(buffer, &mut offset);
    let schema_format = get_u32(buffer, &mut offset);
    let default_page_cache_size = get_u32(buffer, &mut offset);
    let largest_btree_root_page = get_u32(buffer, &mut offset);

    let text_encoding_byte = get_u32(buffer, &mut offset);
    if text_encoding_byte != 1 && text_encoding_byte != 2 && text_encoding_byte != 3 {
        return raise_error(format!("Invalid Text Encoding {}", text_encoding_byte));
    }
    let text_encoding = match text_encoding_byte {
        1 => TextEncoding::UTF8,
        2 => TextEncoding::UTF16le,
        3 => TextEncoding::UTF16be,
        _ => unreachable!(),
    };

    let user_version = get_u32(buffer, &mut offset);
    let incremental_vacuum_mode = get_u32(buffer, &mut offset);
    let application_id = get_u32(buffer, &mut offset);

    let reserved_space_slice = buffer[offset..offset + 20].try_into().unwrap();
    offset += 20;

    let version_valid_for = get_u32(buffer, &mut offset);
    let sqlite_version = get_u32(buffer, &mut offset);

    // Construct the header struct
    Ok(DBHeader {
        header_string: *HEADER_STRING,
        page_size,
        write_version,
        read_version,
        reserved_space_size,
        max_payload_fraction: 64,
        min_payload_fraction: 32,
        leaf_payload_fraction: 32,
        file_change_counter,
        db_size_in_pages: database_size_pages,
        freelist_trunk_page: first_freelist_trunk_page,
        total_freelist_pages: number_of_freelist_pages,
        schema_cookie,
        schema_format_number: schema_format,
        default_page_cache_size,
        largest_btree_root_page,
        text_encoding,
        user_version,
        incremental_vacuum_mode,
        application_id,
        reserved_space: reserved_space_slice,
        version_valid_for,
        sqlite_version,
    })
}

fn raise_error(msg: String) -> Result<DBHeader, DBErrors> {
    Err(DBErrors::InvalidFileHeader(format!(
        "Invalid Header String: {}",
        msg
    )))
}

// Page Parser
//pub fn parse_page(buffer: &[u8], page_number: u32) -> Result<Page, DBErrors> {
//let cursor = 0;
//if page_number == 1 {
//let file_header = parse_file_header(buffer);
//cursor += 100;
//}

//Ok(Page)
//}

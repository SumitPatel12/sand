use super::{
    dbfileheader::{DBHeader, TextEncoding},
    errors::DBErrors,
    page::{
        InteriorPageHeader, LeafPageHeader, PageHeader, PageType, INTERIOR_PAGE_HEADER_SIZE,
        LEAF_PAGE_HEADER_SIZE,
    },
};
use std::{convert::TryInto, fs::File, io::Read};

// File header parser
pub const HEADER_SIZE_BYTES: usize = 100;
const HEADER_STRING: &'static [u8; 16] = b"SQLite format 3\0";
const MAX_PAGE_SIZE: u32 = 65536;
// The static offsets taken from the SQLite documentation
const HEADER_OFFSETS: [usize; 23] = [
    0, 16, 18, 19, 20, 21, 22, 23, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60, 64, 68, 72, 92, 96,
];

fn get_u8(buffer: &[u8], offset: &mut usize) -> u8 {
    let val = buffer[*offset];
    *offset += 1;
    val
}

fn get_u16(buffer: &[u8], offset: &mut usize) -> u16 {
    let val = u16::from_be_bytes(buffer[*offset..*offset + 2].try_into().unwrap());
    *offset += 2;
    val
}

fn get_u32(buffer: &[u8], offset: &mut usize) -> u32 {
    let val = u32::from_be_bytes(buffer[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    val
}

pub fn parse_file_header(buffer: &[u8]) -> Result<DBHeader, DBErrors> {
    let header_string;
    if !buffer.starts_with(HEADER_STRING) {
        header_string = String::from_utf8_lossy(&buffer[..HEADER_STRING.len()]).to_string();
        return Err(DBErrors::InvalidFileHeader(format!(
            "Invalid Header String: {}",
            header_string
        )));
    }

    let mut offset = 16; // Start after HEADER_STRING

    let page_size = match u16::from_be_bytes(buffer[offset..offset + 2].try_into().unwrap()) {
        1 => MAX_PAGE_SIZE,
        page_size_bytes => page_size_bytes as u32,
    };
    if !page_size.is_power_of_two() {
        return Err(DBErrors::InvalidFileHeader(format!(
            "Page size is not a multiple of 2: {}",
            page_size
        )));
    }
    offset += 2;

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
        return Err(DBErrors::InvalidFileHeader(format!(
            "Invalid Text encoding: {}",
            text_encoding_byte
        )));
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

pub fn parse_page_header(file: &mut File) -> Result<PageHeader, DBErrors> {
    let mut offset = 0;
    let mut page_type_buffer = [0u8; 1];
    let mut is_interior_page = false;
    let _ = file.read_exact(&mut page_type_buffer);

    let page_type = match page_type_buffer[0] {
        2 => {
            is_interior_page = true;
            PageType::IndexInterior
        }
        5 => {
            is_interior_page = true;
            PageType::TableInterior
        }
        10 => PageType::IndexLeaf,
        13 => PageType::TableLeaf,
        n => {
            return Err(DBErrors::InvalidPageHeader(format!(
                "Invalid Page Type: {}",
                n
            )))
        }
    };

    let mut buffer = vec![
        0u8;
        if is_interior_page {
            INTERIOR_PAGE_HEADER_SIZE - 1
        } else {
            LEAF_PAGE_HEADER_SIZE - 1
        }
    ];
    let _ = file.read_exact(&mut buffer);

    let first_freeblock = get_u16(&buffer, &mut offset);
    let cell_count = get_u16(&buffer, &mut offset);
    let cell_content_offset = get_u16(&buffer, &mut offset);
    let fragmented_bytes_count = get_u8(&buffer, &mut offset);

    if is_interior_page {
        let right_most_pointer = get_u32(&buffer, &mut offset);
        return Ok(PageHeader::InteriorPageHeader(InteriorPageHeader {
            page_type,
            first_freeblock,
            cell_count,
            cell_content_offset,
            fragmented_bytes_count,
            right_most_pointer,
        }));
    }

    Ok(PageHeader::LeafPageHeader(LeafPageHeader {
        page_type,
        first_freeblock,
        cell_count,
        cell_content_offset,
        fragmented_bytes_count,
    }))
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

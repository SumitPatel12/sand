use super::{
    dbfileheader::{DBHeader, TextEncoding},
    errors::DBError,
    page::{InteriorPageHeader, LeafPageHeader, PageCell, PageHeader, PageType, TableLeafCell},
    varint::get_varint,
};
use std::convert::TryInto;
use std::str;

// File header parser
pub const HEADER_SIZE_BYTES: usize = 100;
const HEADER_STRING: &'static [u8; 16] = b"SQLite format 3\0";
const MAX_PAGE_SIZE: u32 = 65536;

fn get_i8(buffer: &[u8], offset: &mut usize) -> i8 {
    let val = buffer[*offset] as i8;
    *offset += 1;
    val
}

fn get_i16(buffer: &[u8], offset: &mut usize) -> i16 {
    let val = i16::from_be_bytes(buffer[*offset..*offset + 2].try_into().unwrap());
    *offset += 2;
    val
}

fn get_i32(buffer: &[u8], offset: &mut usize) -> i32 {
    let val = i32::from_be_bytes(buffer[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    val
}

fn get_i64(buffer: &[u8], offset: &mut usize) -> i64 {
    let val = i64::from_be_bytes(buffer[*offset..*offset + 8].try_into().unwrap());
    *offset += 8;
    val
}

fn get_f64(buffer: &[u8], offset: &mut usize) -> f64 {
    let val = f64::from_be_bytes(buffer[*offset..*offset + 8].try_into().unwrap());
    *offset += 8;
    val
}

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

pub fn parse_file_header(buffer: &[u8]) -> Result<DBHeader, DBError> {
    let header_string;
    if !buffer.starts_with(HEADER_STRING) {
        header_string = String::from_utf8_lossy(&buffer[..HEADER_STRING.len()]).to_string();
        return Err(DBError::InvalidFileHeader(format!(
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
        return Err(DBError::InvalidFileHeader(format!(
            "Page size is not a multiple of 2: {}",
            page_size
        )));
    }
    offset += 2;

    // Extract remaining fields
    let write_version = get_u8(buffer, &mut offset);
    let read_version = get_u8(buffer, &mut offset);
    let reserved_space_size = get_u8(buffer, &mut offset);
    let max_payload_fraction = get_u8(buffer, &mut offset);
    let min_payload_fraction = get_u8(buffer, &mut offset);
    let leaf_payload_fraction = get_u8(buffer, &mut offset);
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
        return Err(DBError::InvalidFileHeader(format!(
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
        max_payload_fraction,
        min_payload_fraction,
        leaf_payload_fraction,
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

pub fn parse_page_header(buffer: &[u8], offset: &mut usize) -> Result<PageHeader, DBError> {
    let mut is_interior_page = false;

    let page_type = match buffer[*offset] {
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
            return Err(DBError::InvalidPageHeader(format!(
                "Invalid Page Type: {}",
                n
            )))
        }
    };
    *offset += 1;

    let first_freeblock = get_u16(&buffer, offset);
    let cell_count = get_u16(&buffer, offset);
    let cell_content_offset = get_u16(&buffer, offset);
    let fragmented_bytes_count = get_u8(&buffer, offset);

    if is_interior_page {
        let right_most_pointer = get_u32(&buffer, offset);
        return Ok(PageHeader::InteriorPageHeader(InteriorPageHeader {
            page_type,
            first_freeblock,
            cell_count,
            cell_content_offset,
            fragmented_bytes_count,
            right_most_pointer,
        }));
    }

    // TODO: Implement Reading Cell Pointer Array Read Logic.

    *offset = cell_content_offset as usize;
    parse_cell(buffer, page_type, offset)?;

    Ok(PageHeader::LeafPageHeader(LeafPageHeader {
        page_type,
        first_freeblock,
        cell_count,
        cell_content_offset,
        fragmented_bytes_count,
    }))
}

pub fn parse_cell(
    buffer: &[u8],
    page_type: PageType,
    offset: &mut usize,
) -> Result<PageCell, DBError> {
    match page_type {
        PageType::TableLeaf => return parse_table_leaf_cell(buffer, offset),
        PageType::TableInterior => return parse_table_leaf_cell(buffer, offset),
        PageType::IndexLeaf => return parse_table_leaf_cell(buffer, offset),
        PageType::IndexInterior => return parse_table_leaf_cell(buffer, offset),
    };
}

fn parse_table_leaf_cell(buffer: &[u8], offset: &mut usize) -> Result<PageCell, DBError> {
    // TODO: Maybe something to not manually update the offset.
    let payload_size = get_varint(&buffer, offset);
    let row_id = get_varint(&buffer, offset);
    read_payload(buffer, offset)?;

    Ok(PageCell::TableLeafCell(TableLeafCell {
        payload_size,
        row_id,
        payload: vec![0u8; 1],
    }))
}

// fn parse_table_interior_cell(file: &mut file) -> Result<PageCell, Box<dyn std::error::Error>> {}

// fn parse_index_leaf_cell(file: &mut file) -> Result<PageCell, Box<dyn std::error::Error>> {}

// fn parse_index_interior_cell(file: &mut file) -> Result<PageCell, Box<dyn std::error::Error>> {}

fn read_payload(buffer: &[u8], offset: &mut usize) -> Result<(), DBError> {
    let header_start_offset = (*offset).clone();
    let header_size = get_varint(&buffer, offset);

    let mut serial_types: Vec<u64> = Vec::new();
    while *offset < (header_start_offset + header_size as usize) {
        let serial_type = get_varint(&buffer, offset);
        serial_types.push(serial_type);
    }

    for serial_type in serial_types {
        match serial_type {
            0 => {
                println!("Null Value");
            }
            1 => {
                let value = get_i8(buffer, offset);
            }
            2 => {
                let value = get_i16(buffer, offset);
            }
            3 => {
                // TODO: Handle i24 somehow.
                // let value = get_i24(buffer, offset);
            }
            4 => {
                let value = get_i32(buffer, offset);
                println!("{}", value);
            }
            5 => {
                // TODO: Handle i48 somehow.
                // let value = get_i48(buffer, offset);
            }
            6 => {
                let value = get_i64(buffer, offset);
            }
            7 => {
                let value = get_f64(buffer, offset);
            }
            8 => {
                let value = 0;
            }
            9 => {
                let value = 1;
            }
            x if x == 10 || x == 11 => {
                println!("Used for SQLite internal stuff!");
            }
            x if x >= 12 && x % 2 == 0 => {
                // TODO: Implement Blob.
                println!("In Read Blob.");
            }
            x if x >= 13 && x % 2 == 1 => {
                let string_size = (x as usize - 13) / 2;
                let payload_string = str::from_utf8(&buffer[*offset..*offset + string_size]);
                *offset += string_size as usize;
                match payload_string {
                    Ok(payload_string) => println!("{} \n", payload_string),
                    Err(e) => println!("Error reading string {}", e),
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

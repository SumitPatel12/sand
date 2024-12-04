// TODO: Typecasting in this project is ugly. Look into better things.
use super::{
    dbfileheader::{DBHeader, TextEncoding},
    errors::DBError,
    page::{InteriorPageHeader, LeafPageHeader, PageCell, PageHeader, PageType, TableLeafCell},
    varint::get_varint,
    SQLiteSchema,
};
use std::str;
use std::{convert::TryInto, fs::File};
use std::{
    io::{Read, Seek},
    u16,
};

// File header parser
pub const HEADER_SIZE_BYTES: usize = 100;
const HEADER_STRING: &'static [u8; 16] = b"SQLite format 3\0";
const MAX_PAGE_SIZE: u32 = 65536;

// TODO: Create a wrapper for all these methods and the buffer and pass that wrapper around maybe.
fn read_i8(buffer: &[u8], offset: &mut usize) -> i8 {
    let val = buffer[*offset] as i8;
    *offset += 1;
    val
}

fn read_i16(buffer: &[u8], offset: &mut usize) -> i16 {
    let val = i16::from_be_bytes(buffer[*offset..*offset + 2].try_into().unwrap());
    *offset += 2;
    val
}

fn read_i32(buffer: &[u8], offset: &mut usize) -> i32 {
    let val = i32::from_be_bytes(buffer[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    val
}

fn read_i64(buffer: &[u8], offset: &mut usize) -> i64 {
    let val = i64::from_be_bytes(buffer[*offset..*offset + 8].try_into().unwrap());
    *offset += 8;
    val
}

fn read_f64(buffer: &[u8], offset: &mut usize) -> f64 {
    let val = f64::from_be_bytes(buffer[*offset..*offset + 8].try_into().unwrap());
    *offset += 8;
    val
}

fn read_u8(buffer: &[u8], offset: &mut usize) -> u8 {
    let val = buffer[*offset];
    *offset += 1;
    val
}

fn read_u16(buffer: &[u8], offset: &mut usize) -> u16 {
    let val = u16::from_be_bytes(buffer[*offset..*offset + 2].try_into().unwrap());
    *offset += 2;
    val
}

fn read_u32(buffer: &[u8], offset: &mut usize) -> u32 {
    let val = u32::from_be_bytes(buffer[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    val
}

// TODO: Create a wrapper for the parer methods and have the page size as a constant.
pub fn parse_file() -> Result<(), Box<dyn std::error::Error>> {
    let db_file_path = "../sand.db";
    let mut file = File::open(db_file_path)?;
    let (page_size, tables) = parse_schema_page(&mut file)?;

    for table in tables {
        println!("Table: {:#?}", table);
        parse_page(&mut file, table.root_page as u64, page_size)?;
    }

    Ok(())
}

// TODO: I know code duplication and what not but I need to get something working before worrying about
// all of that.
pub fn parse_schema_page(
    file: &mut File,
) -> Result<(u32, Vec<SQLiteSchema>), Box<dyn std::error::Error>> {
    let mut buffer = [0u8; 100];
    let mut offset = 100;

    file.read_exact(&mut buffer)?;

    let file_header = parse_db_file_header(&buffer)?;
    println!("{:#?}", file_header);

    let mut page_buffer = vec![0u8; file_header.page_size as usize];
    file.seek(std::io::SeekFrom::Start(0))?;
    file.read_exact(&mut page_buffer)?;

    let page_header = parse_page_header(&page_buffer, &mut offset)?;
    println!("{:#?}", page_header);

    // TODO: Maybe check if we actually have a payload before intializing this.
    let mut tables: Vec<SQLiteSchema> = Vec::new();

    for cell_offset in get_cell_pointers(&page_buffer, &mut offset, page_header) {
        offset = cell_offset as usize;
        let payload_size = get_varint(&page_buffer, &mut offset);
        let row_id = get_varint(&page_buffer, &mut offset);

        println!("Payload: {}, Row Id: {}", payload_size, row_id);
        // TODO: This only works for leaf nodes better get this sorted out as well.
        tables.push(read_sqlite_schema_cell_payload(&page_buffer, &mut offset)?);
    }

    Ok((file_header.page_size, tables))
}

pub fn parse_db_file_header(buffer: &[u8]) -> Result<DBHeader, DBError> {
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
    let write_version = read_u8(buffer, &mut offset);
    let read_version = read_u8(buffer, &mut offset);
    let reserved_space_size = read_u8(buffer, &mut offset);
    let max_payload_fraction = read_u8(buffer, &mut offset);
    let min_payload_fraction = read_u8(buffer, &mut offset);
    let leaf_payload_fraction = read_u8(buffer, &mut offset);
    let file_change_counter = read_u32(buffer, &mut offset);
    let database_size_pages = read_u32(buffer, &mut offset);
    let first_freelist_trunk_page = read_u32(buffer, &mut offset);
    let number_of_freelist_pages = read_u32(buffer, &mut offset);
    let schema_cookie = read_u32(buffer, &mut offset);
    let schema_format = read_u32(buffer, &mut offset);
    let default_page_cache_size = read_u32(buffer, &mut offset);
    let largest_btree_root_page = read_u32(buffer, &mut offset);

    let text_encoding_byte = read_u32(buffer, &mut offset);
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

    let user_version = read_u32(buffer, &mut offset);
    let incremental_vacuum_mode = read_u32(buffer, &mut offset);
    let application_id = read_u32(buffer, &mut offset);

    let reserved_space_slice = buffer[offset..offset + 20].try_into().unwrap();
    offset += 20;

    let version_valid_for = read_u32(buffer, &mut offset);
    let sqlite_version = read_u32(buffer, &mut offset);

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

pub fn parse_page(
    file: &mut File,
    page: u64,
    page_size: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Look into this page as usize thing u64 as u32 could happen, which is not ideal
    let mut page_buffer = vec![0u8; page_size as usize];
    let mut offset = 0;

    file.seek(std::io::SeekFrom::Start(page_size as u64 * (page - 1)))?;
    file.read_exact(&mut page_buffer)?;

    let page_header = parse_page_header(&page_buffer, &mut offset)?;
    println!("Page {}\nPage Header: {:#?}", page, page_header);

    let page_type = match page_header {
        PageHeader::LeafPageHeader(l) => l.page_type,
        PageHeader::InteriorPageHeader(i) => i.page_type,
    };

    for cell_offset in get_cell_pointers(&page_buffer, &mut offset, page_header) {
        // TODO: Bruh this is really wrong you gotta look at the docs and see if this is really the
        // way. Both rust and sqlite docs.
        offset = cell_offset as usize;
        parse_cell(&page_buffer, page_type, &mut offset)?;
    }

    Ok(())
}

pub fn parse_page_header(
    buffer: &[u8],
    offset: &mut usize,
) -> Result<PageHeader, Box<dyn std::error::Error>> {
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
            return Err(Box::new(DBError::InvalidPageHeader(format!(
                "Invalid Page Type: {}",
                n
            ))))
        }
    };
    *offset += 1;

    let first_freeblock = read_u16(&buffer, offset);
    let cell_count = read_u16(&buffer, offset);
    let cell_content_offset = read_u16(&buffer, offset);
    let fragmented_bytes_count = read_u8(&buffer, offset);

    if is_interior_page {
        let right_most_pointer = read_u32(&buffer, offset);
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

pub fn parse_cell(
    buffer: &[u8],
    page_type: PageType,
    offset: &mut usize,
) -> Result<PageCell, Box<dyn std::error::Error>> {
    match page_type {
        PageType::TableLeaf => {
            println!("Table Leaf Cell");
            return parse_table_leaf_cell(buffer, offset);
        }
        PageType::TableInterior => {
            println!("Table Interior");
            return parse_table_leaf_cell(buffer, offset);
        }
        PageType::IndexLeaf => {
            println!("Index Leaf");
            return parse_table_leaf_cell(buffer, offset);
        }
        PageType::IndexInterior => {
            println!("Index Interior");
            return parse_table_leaf_cell(buffer, offset);
        }
    };
}

// TODO: Return type.
fn parse_table_leaf_cell(
    buffer: &[u8],
    offset: &mut usize,
) -> Result<PageCell, Box<dyn std::error::Error>> {
    // TODO: Maybe something to not manually update the offset.
    let payload_size = get_varint(&buffer, offset);
    let row_id = get_varint(&buffer, offset);
    println!("Payload: {}, Row Id {}", payload_size, row_id);
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

// TODO: Think of some dynamic struct creation for columns.
// Also remove that page offset < 4096 thing. It's working now and I need to see if I can read the correct data
// but this is not a solution but an ugly hack.
// Add out of bounds check.
fn read_payload(buffer: &[u8], offset: &mut usize) -> Result<(), Box<dyn std::error::Error>> {
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
                let value = read_i8(buffer, offset);
            }
            2 => {
                let value = read_i16(buffer, offset);
            }
            3 => {
                // TODO: Handle i24 somehow.
                // let value = get_i24(buffer, offset);
            }
            4 => {
                let value = read_i32(buffer, offset);
                println!("{}", value);
            }
            5 => {
                // TODO: Handle i48 somehow.
                // let value = get_i48(buffer, offset);
            }
            6 => {
                let value = read_i64(buffer, offset);
            }
            7 => {
                let value = read_f64(buffer, offset);
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

pub fn read_sqlite_schema_cell_payload(
    buffer: &[u8],
    offset: &mut usize,
) -> Result<SQLiteSchema, Box<dyn std::error::Error>> {
    let header_start_offset = (*offset).clone();
    let header_size = get_varint(&buffer, offset);

    let mut serial_types: Vec<u64> = Vec::new();
    while *offset < (header_start_offset + header_size as usize) {
        let serial_type = get_varint(&buffer, offset);
        serial_types.push(serial_type);
    }

    let mut string_size = (serial_types[0] as usize - 13) / 2;
    let schema_type = str::from_utf8(&buffer[*offset..*offset + string_size])?;
    *offset += string_size;

    string_size = (serial_types[1] as usize - 13) / 2;
    let name = str::from_utf8(&buffer[*offset..*offset + string_size])?;
    *offset += string_size;

    string_size = (serial_types[2] as usize - 13) / 2;
    let table_name = str::from_utf8(&buffer[*offset..*offset + string_size])?;
    *offset += string_size;

    let root_page = read_i8(buffer, offset);

    string_size = (serial_types[4] as usize - 13) / 2;
    let sql = str::from_utf8(&buffer[*offset..*offset + string_size])?;
    *offset += string_size;
    Ok(SQLiteSchema {
        schema_type: schema_type.to_string(),
        name: name.to_string(),
        table_name: table_name.to_string(),
        root_page,
        sql: sql.to_lowercase(),
    })
}

fn get_cell_pointers(buffer: &[u8], offset: &mut usize, page_header: PageHeader) -> Vec<u16> {
    let cell_count = page_header.cell_count();
    let mut cell_pointers = vec![0u16; cell_count as usize];
    for i in 0..cell_count {
        cell_pointers[i as usize] = read_u16(&buffer, offset);
    }

    cell_pointers
}

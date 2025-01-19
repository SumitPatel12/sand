use crate::page::errors::DBError;
use anyhow::{anyhow, Result};
use core::fmt;
use std::convert::TryFrom;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    usize,
};

// Database header size in bytes.
pub const DB_HEADER_SIZE: usize = 100;

// The constant file header.
pub const HEADER_STRING: &'static [u8; 16] = b"SQLite format 3\0";

#[derive(Debug, Clone)]
pub enum TextEncoding {
    UTF8,
    UTF16le,
    UTF16be,
}

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
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DBHeader {
    header_string: [u8; 16],
    pub page_size: u16,
    write_version: u8,
    read_version: u8,
    reserved_space: u8,
    max_payload_fraction: u8,
    min_payload_fraction: u8,
    min_leaf_fraction: u8,
    change_counter: u32,
    db_size_in_pages: u32,
    freelist_trunk_page: u32,
    freelist_pages: u32,
    schema_cookie: u32,
    schema_format: u32,
    default_cache_page_size: u32,
    vacuum: u32,
    text_encoding: TextEncoding,
    user_version: u32,
    incremental_vacuum: u32,
    application_id: u32,
    reserved: [u8; 20],
    version_valid_for: u32,
    version_number: u32,
}

pub fn read_db_header(file: &mut File) -> Result<DBHeader> {
    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(100, 0);
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut buffer)?;

    let mut header_string = [0u8; 16];
    let mut reserved = [0u8; 20];
    let page_size = u16::from_be_bytes([buffer[16], buffer[17]]);
    let text_encoding_byte = u32::from_be_bytes([buffer[56], buffer[57], buffer[58], buffer[59]]);

    header_string.copy_from_slice(&buffer[0..16]);
    reserved.copy_from_slice(&buffer[72..92]);

    if !buffer.starts_with(HEADER_STRING) {
        return Err(anyhow!(DBError::InvalidFileHeader(format!(
            "Invalid Header String: {}",
            String::from_utf8_lossy(&buffer[..HEADER_STRING.len()]).to_string()
        ))));
    }

    if !page_size.is_power_of_two() {
        return Err(anyhow!(DBError::InvalidFileHeader(format!(
            "Page size must be a power of 2: {}",
            page_size
        ))));
    }

    if text_encoding_byte != 1 && text_encoding_byte != 2 && text_encoding_byte != 3 {
        return Err(anyhow!(DBError::InvalidFileHeader(format!(
            "Invalid text encoding: {}",
            text_encoding_byte
        ))));
    }

    let header = DBHeader {
        header_string,
        page_size,
        write_version: buffer[18],
        read_version: buffer[19],
        reserved_space: buffer[20],
        max_payload_fraction: buffer[21],
        min_payload_fraction: buffer[22],
        min_leaf_fraction: buffer[23],
        change_counter: u32::from_be_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]),
        db_size_in_pages: u32::from_be_bytes([buffer[28], buffer[29], buffer[30], buffer[31]]),
        freelist_trunk_page: u32::from_be_bytes([buffer[32], buffer[33], buffer[34], buffer[35]]),
        freelist_pages: u32::from_be_bytes([buffer[36], buffer[37], buffer[38], buffer[39]]),
        schema_cookie: u32::from_be_bytes([buffer[40], buffer[41], buffer[42], buffer[43]]),
        schema_format: u32::from_be_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]),
        default_cache_page_size: u32::from_be_bytes([
            buffer[48], buffer[49], buffer[50], buffer[51],
        ]),
        vacuum: u32::from_be_bytes([buffer[52], buffer[53], buffer[54], buffer[55]]),
        text_encoding: match text_encoding_byte {
            1 => TextEncoding::UTF8,
            2 => TextEncoding::UTF16le,
            3 => TextEncoding::UTF16be,
            _ => unreachable!(),
        },
        user_version: u32::from_be_bytes([buffer[60], buffer[61], buffer[62], buffer[63]]),
        incremental_vacuum: u32::from_be_bytes([buffer[64], buffer[65], buffer[66], buffer[67]]),
        application_id: u32::from_be_bytes([buffer[68], buffer[69], buffer[70], buffer[71]]),
        reserved,
        version_valid_for: u32::from_be_bytes([buffer[92], buffer[93], buffer[94], buffer[95]]),
        version_number: u32::from_be_bytes([buffer[96], buffer[97], buffer[98], buffer[99]]),
    };

    Ok(header)
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct BTreePageHeader {
    page_type: PageType,
    first_freeblock_offset: u16,
    cell_count: u16,
    cell_content_area: u16,
    number_of_fragmented_free_bytes: u8,
    right_most_pointer: Option<u32>,
}

#[derive(Debug, PartialEq)]
pub enum PageType {
    IndexInteriorPage = 2,
    TableInteriorPage = 5,
    IndexLeafPage = 10,
    TableLeafPage = 13,
}

#[derive(Debug)]
pub struct BTreePage {
    pub header: BTreePageHeader,
    pub cells: Vec<BTreeCell>,
}

// TODO: Look into how databases handle I/O, passing the file descriptor around like a maniac is
// not gonna cut it.
pub fn read_page(file: &mut File, page_size: usize, page_index: usize) -> Result<BTreePage> {
    // TODO: Implement some IO maybe?
    let mut page: Vec<u8> = Vec::new();
    page.resize(page_size, 0);
    file.seek(SeekFrom::Start(((page_index - 1) * page_size) as u64))?;
    file.read_exact(&mut page)?;

    let mut offset = if page_index == 1 { DB_HEADER_SIZE } else { 0 };
    println!("Offset for reading Page Header: {}", offset);

    let mut header = BTreePageHeader {
        page_type: page[offset].try_into()?,
        first_freeblock_offset: u16::from_be_bytes([page[offset + 1], page[offset + 2]]),
        cell_count: u16::from_be_bytes([page[offset + 3], page[offset + 4]]),
        cell_content_area: u16::from_be_bytes([page[offset + 5], page[offset + 6]]),
        number_of_fragmented_free_bytes: page[offset + 7],
        right_most_pointer: None,
    };
    offset += 8;

    if header.page_type == PageType::TableInteriorPage
        || header.page_type == PageType::IndexInteriorPage
    {
        header.right_most_pointer = Some(u32::from_be_bytes([
            page[offset],
            page[offset + 1],
            page[offset + 2],
            page[offset + 3],
        ]));
        offset += 4;
    }

    let mut cells = Vec::new();

    for _ in 0..header.cell_count {
        let cell_pointer = u16::from_be_bytes([page[offset], page[offset + 1]]);
        offset += 2;

        let cell = read_cell(&page, &header.page_type, cell_pointer as usize)?;

        cells.push(cell);
    }

    // I asked around and turns out if you know its gonna be static is better to directly use the
    // exact positions rather than incrementing offsets.
    Ok(BTreePage {
        // TODO: Get rid of dummy values
        header,
        cells,
    })
}

#[derive(Debug)]
pub enum BTreeCell {
    TableInteriorCell(TableInteriorCell),
    TableLeafCell(TableLeafCell),
    IndexInteriorCell(IndexInteriorCell),
    IndexLeafCell(IndexLeafCell),
}

#[derive(Debug)]
pub struct TableInteriorCell {
    pub left_child_page: u32,
    pub rowid: u64,
}

// TODO: Probably should not directly use the parsed record in here.
// Maybe use the raw bytes as a vector and parse it as required? Anyway do more reasearch on how
// SQLite does this.
#[derive(Debug)]
pub struct TableLeafCell {
    pub row_id: u64,
    pub record: Record,
    pub first_overflow_page: Option<u32>,
}

// TODO: Probably should not directly use the parsed record in here.
// Maybe use the raw bytes as a vector and parse it as required? Anyway do more reasearch on how
// SQLite does this.
#[derive(Debug)]
pub struct IndexInteriorCell {
    pub left_child_page: u32,
    pub record: Record,
    pub first_overflow_page: Option<u32>,
}

#[derive(Debug)]
pub struct IndexLeafCell {
    pub payload: Vec<u8>,
    pub first_overflow_page: Option<u32>,
}

// ChatGPT says you can use try_into rather than writing `try_get_page_type`. Interesting to konw.
impl TryFrom<u8> for PageType {
    // NOTE: Turns out if you want to use anyhow Result here you need to define the error type as
    // anyhow::Error.
    type Error = anyhow::Error;

    fn try_from(page_type_value: u8) -> Result<PageType> {
        println!("Trying page type for: {}", page_type_value);
        match page_type_value {
            2 => Ok(Self::IndexInteriorPage),
            5 => Ok(Self::TableInteriorPage),
            10 => Ok(Self::IndexLeafPage),
            13 => Ok(Self::TableLeafPage),
            _ => Err(anyhow!("Invalid Page Type: {}", page_type_value)),
        }
    }
}

fn read_cell(page: &[u8], page_type: &PageType, offset: usize) -> Result<BTreeCell> {
    match page_type {
        PageType::IndexInteriorPage => todo!(),
        PageType::TableInteriorPage => todo!(),
        PageType::IndexLeafPage => todo!(),
        PageType::TableLeafPage => {
            let mut offset = offset;
            let (payload_size, varint_size) = read_varint(&page[offset..])?;
            offset += varint_size;

            let (row_id, varint_size) = read_varint(&page[offset..])?;
            offset += varint_size;

            let payload = &page[offset..offset + payload_size as usize];
            Ok(BTreeCell::TableLeafCell(TableLeafCell {
                row_id,
                record: read_record(&payload)?,
                first_overflow_page: None,
            }))
        }
    }
}

#[derive(Debug)]
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
}

#[allow(dead_code)]
pub struct Record {
    values: Vec<Value>,
}

impl fmt::Debug for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Record: {{")?;

        for value in &self.values {
            writeln!(f, " {:?}", value)?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug)]
pub enum SerialType {
    Null,
    I8,
    I16,
    I24,
    I32,
    I48,
    I64,
    F64,
    Int0,
    Int1,
    Blob(usize),
    String(usize),
}

// Same thing try from is better suited than `try_get_serial_type`.
impl TryFrom<u64> for SerialType {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self> {
        match value {
            0 => Ok(Self::Null),
            1 => Ok(Self::I8),
            2 => Ok(Self::I16),
            3 => Ok(Self::I24),
            4 => Ok(Self::I32),
            5 => Ok(Self::I48),
            6 => Ok(Self::I64),
            7 => Ok(Self::F64),
            8 => Ok(Self::Int0),
            9 => Ok(Self::Int1),
            n if value > 12 && value % 2 == 0 => Ok(Self::Blob(((n - 12) / 2) as usize)),
            n if value > 13 && value % 2 == 1 => Ok(Self::String(((n - 13) / 2) as usize)),
            _ => Err(anyhow!(DBError::InvalidSerialType(value))),
        }
    }
}

fn read_record(payload: &[u8]) -> Result<Record> {
    let mut offset = 0;
    let (header_size, varint_size) = read_varint(&payload[offset..])?;
    offset += varint_size;

    let mut header_size = header_size as usize - varint_size;
    let mut serial_types = Vec::new();

    while header_size > 0 {
        let (serial_type, varint_size) = read_varint(&payload[offset..])?;
        let serial_type = SerialType::try_from(serial_type)?;
        offset += varint_size;
        serial_types.push(serial_type);

        header_size -= varint_size;
    }

    let mut values = Vec::new();

    for serial_type in serial_types {
        let (value, size) = read_value(&payload[offset..], serial_type)?;
        offset += size;
        values.push(value);
    }

    Ok(Record { values })
}

fn read_value(buffer: &[u8], serial_type: SerialType) -> Result<(Value, usize)> {
    match serial_type {
        SerialType::Null => Ok((Value::Null, 0)),
        SerialType::I8 => Ok((Value::Integer(buffer[0] as i64), 1)),
        SerialType::I16 => Ok((
            Value::Integer(i16::from_be_bytes([buffer[0], buffer[1]]) as i64),
            2,
        )),
        SerialType::I24 => Ok((
            Value::Integer(i32::from_be_bytes([0, buffer[0], buffer[1], buffer[2]]) as i64),
            3,
        )),
        SerialType::I32 => Ok((
            Value::Integer(i32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as i64),
            4,
        )),
        SerialType::I48 => Ok((
            Value::Integer(i64::from_be_bytes([
                0, 0, buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5],
            ])),
            6,
        )),
        SerialType::I64 => Ok((
            Value::Integer(i64::from_be_bytes([
                buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                buffer[7],
            ])),
            8,
        )),
        SerialType::F64 => Ok((
            Value::Float(f64::from_be_bytes([
                buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                buffer[7],
            ])),
            8,
        )),
        SerialType::Int0 => Ok((Value::Integer(0), 0)),
        SerialType::Int1 => Ok((Value::Integer(1), 0)),
        SerialType::Blob(n) => Ok((Value::Blob(buffer[0..n].to_vec()), n)),
        SerialType::String(n) => {
            let value = String::from_utf8(buffer[0..n].to_vec())?;
            Ok((Value::Text(value), n))
        }
    }
}

pub fn read_varint(buffer: &[u8]) -> Result<(u64, usize)> {
    let mut size = 0;
    let mut decoded_varint = 0;
    let mut offset = 0;

    while size < 8 && buffer[offset] >= 0x80 {
        decoded_varint |= u64::from(buffer[offset]) & 0x7F;
        decoded_varint <<= 7;
        offset += 1;
        size += 1;
    }

    decoded_varint |= u64::from(buffer[offset]) & 0x7F;
    size += 1;
    Ok((decoded_varint, size))
}

// TODO: Read documentation of other dbs for encoding and make a more generic function.
// Implement this as well
pub fn encode_varint(_value: u64) -> Vec<u8> {
    vec![0u8; 1]
}

#[cfg(test)]
mod tests {
    use crate::page::file_structures::read_varint;

    #[test]
    fn decode_varint_test() {
        let result = read_varint(&[0x82, 0x2C]);
        match result {
            Ok(value) => assert_eq!(value, (300 as u64, 2)),
            Err(..) => unreachable!(),
        }

        let result = read_varint(&[0x81, 0x07]);
        match result {
            Ok(value) => assert_eq!(value, (135 as u64, 2)),
            Err(..) => unreachable!(),
        }
    }
}

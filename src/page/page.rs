use super::dbfileheader::DBHeader;

pub const PAGE_HEADER_FIRST_FREEBLOCK_OFFSET: u8 = 1;
pub const PAGE_HEADER_CELL_COUNT_OFFSET: u16 = 3;
pub const PAGE_HEADER_CELL_CONTENT_AREA_START_OFFSET: u16 = 5;
pub const PAGE_HEADER_FRAGMENTED_BYTES_COUNT_OFFSET: u16 = 7;
pub const PAGE_HEADER_RIGHT_MOST_POINTER_OFFSET: u32 = 8;
pub const LEAF_PAGE_HEADER_SIZE: usize = 8;
pub const INTERIOR_PAGE_HEADER_SIZE: usize = 12;

#[derive(Debug)]
pub enum Page {
    TableLeaf(TableLeafPage),
    TableInterior,
    IndexLeaf,
    IndexInterior,
}

#[derive(Debug)]
pub struct TableLeafPage {
    pub dbheader: DBHeader,
    pub header: PageHeader,
    // Maybe I don't need to include this.
    // pub unallocated_space: Vec<u8>,
    pub cell_pointers: Vec<u16>,
    pub cells: Vec<TableLeafCell>,
    // This might not be needed either. We will see.
    // pub reserved_region: Vec<u8>,
}

#[derive(Debug, Copy, Clone)]
pub enum PageHeader {
    LeafPageHeader(LeafPageHeader),
    InteriorPageHeader(InteriorPageHeader),
}

#[derive(Debug, Copy, Clone)]
pub struct LeafPageHeader {
    pub page_type: PageType,
    pub first_freeblock: u16,
    pub cell_count: u16,
    pub cell_content_offset: u16,
    pub fragmented_bytes_count: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct InteriorPageHeader {
    pub page_type: PageType,
    pub first_freeblock: u16,
    pub cell_count: u16,
    pub cell_content_offset: u16,
    pub fragmented_bytes_count: u8,
    pub right_most_pointer: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum PageType {
    TableLeaf,
    TableInterior,
    IndexLeaf,
    IndexInterior,
}

#[derive(Debug, Clone)]
pub struct TableLeafCell {
    pub size: i64,
    pub row_id: i64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TableInteriorCell {
    pub left_child_pointer: i64,
    pub row_id: i64,
}

#[derive(Debug, Clone)]
pub struct IndexLeafCell {
    pub size_including_overflow: i64,
    pub payload: Vec<u8>,
    pub overflow_page_offset: u32,
}

#[derive(Debug, Clone)]
pub struct IndexInteriorCell {
    pub left_child_pointer: i64,
    pub size_including_overflow: i64,
    pub payload: Vec<u8>,
    pub overflow_page_offset: u32,
}

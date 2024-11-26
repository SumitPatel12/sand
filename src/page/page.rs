use super::dbfileheader::DBHeader;

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
    TableLeafHeader(TableLeafPageHeader),
    TableInteriorHeader,
    IndexLeafHeader,
    IndexInteriorHeader,
}

#[derive(Debug, Copy, Clone)]
pub struct TableLeafPageHeader {
    pub page_type: PageType,
    pub first_freeblock: u16,
    pub cell_count: u16,
    pub cell_content_offset: u16,
    pub fragmented_bytes_coutn: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct TableInteriorPageHeader {
    pub page_type: PageType,
    pub first_freeblock: u16,
    pub cell_count: u16,
    pub cell_content_offset: u16,
    pub fragmented_bytes_coutn: u8,
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

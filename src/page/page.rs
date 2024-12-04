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

/*
* 1. Database header is present for the first page in the DB. It includes the DB metadata and links
* the table data maintained by sqlite.
* 2. Page header, gives the general layout of the page.
* 3. Immediately after the page header we have cell pointer array. The pointers are arranged in key
*    order, left-most cell has the smallest key and is first, right-most cell has the largest key
*    and is last.
* 4. Unallocated space, the space between cell pointers and the cell content area.
* 5. Cell content area is where the actual cell content starts. For leaf pages it contains the
*    actual table data.
* 6. Then comes the reserved space, it is normally 0, but can be non-zero as well. The usable space
*    of the page is page-size minus the reserved space.
* Then there is a concept of freeblocks.
*/
#[derive(Debug)]
pub struct TableLeafPage {
    pub dbheader: DBHeader,
    pub header: PageHeader,
    // Maybe I don't need to include this.
    // pub unallocated_space: Vec<u8>,
    pub cell_pointers: Vec<u16>,
    // Maybe I don't need this here.
    // pub cells: Vec<TableLeafCell>,
}

#[derive(Debug, Copy, Clone)]
pub enum PageHeader {
    LeafPageHeader(LeafPageHeader),
    InteriorPageHeader(InteriorPageHeader),
}

/*
* 1. Page type is either index interior, table interior, index leaf, or table leaf, indicated by 2,
*    5, 10, and 13 respectively.
* 2. 1 byte integer that gives the first freeblick page. 0 if there are no freeblocks. It is used
*    to keep track of unallocated space within the page and are organized as chains. It is at leat
*    4 bytes in size.
* 3. The number of cells on the page.
* 4. 2 byte integer, start of the cell content area.
* 5. 1 byte integer, the number of fragmantd free bytes.
* 6. For interior pages only, the right most pointer.
*/
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
pub enum PageCell {
    TableLeafCell(TableLeafCell),
    TableInteriorCell(TableInteriorCell),
    IndexLeafCell(IndexLeafCell),
    IndexInteriorCell(IndexInteriorCell),
}

/*
* Both 1 and 2 are varints.
* 1. The size of the payload that the cell contains.
* 2. row_id.
* 3. Actual Payload.
*    - The payload starts with a header followed by the actual values.
*    - The header:
*       - Starts with a varint indicating the size of the header including the size varint.
*       - Followed by varints (serial types), these are used to identify the data type of each
*         column.
* 4. If the payload does not fit on the page then we have the offset of the overflow page.
*/
#[derive(Debug, Clone)]
pub struct TableLeafCell {
    pub payload_size: u64,
    pub row_id: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TableInteriorCell {
    pub left_child_pointer: u64,
    pub row_id: u64,
}

#[derive(Debug, Clone)]
pub struct IndexLeafCell {
    pub size_including_overflow: u64,
    pub payload: Vec<u8>,
    pub overflow_page_offset: u32,
}

#[derive(Debug, Clone)]
pub struct IndexInteriorCell {
    pub left_child_pointer: u64,
    pub size_including_overflow: u64,
    pub payload: Vec<u8>,
    pub overflow_page_offset: u32,
}

impl PageHeader {
    pub fn cell_count(&self) -> u16 {
        match self {
            PageHeader::LeafPageHeader(l) => l.cell_count,
            PageHeader::InteriorPageHeader(i) => i.cell_count,
        }
    }
}

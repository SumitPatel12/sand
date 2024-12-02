pub mod page;

use std::fs::File;
use std::io::{Read, Seek};
use std::usize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_file_path = "../sand.db";
    let mut file = File::open(db_file_path)?;
    let mut buffer = [0u8; 100];
    let mut offset: usize = 100;
    file.read_exact(&mut buffer)?;

    let file_header = page::page_parser::parse_file_header(&buffer)?;
    println!("{:#?}", file_header);

    let mut page_buffer = vec![0u8; file_header.page_size as usize];
    file.seek(std::io::SeekFrom::Start(0))?;
    file.read_exact(&mut page_buffer)?;

    let page_header = page::page_parser::parse_page_header(&page_buffer, &mut offset)?;
    println!("{:#?}", page_header);

    Ok(())
}

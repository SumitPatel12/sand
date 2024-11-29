pub mod page;

use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_file_path = "../sand.db";
    let mut file = File::open(db_file_path)?;
    let mut buffer = [0u8; 100];
    file.read_exact(&mut buffer)?;

    let file_header = page::page_parser::parse_file_header(&buffer)?;
    println!("{:#?}", file_header);

    let page_header = page::page_parser::parse_page_header(&mut file)?;
    println!("{:#?}", page_header);

    Ok(())
}

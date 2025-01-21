// All of the todos are left by me, just that sometimes I forget or sometimes I address myself in
// third person, don't sweat it.
use anyhow::Result;
use page::Database;

pub mod page;

// TODO: At some point remove the allow dead code thingy.
fn main() -> Result<()> {
    let db_file_path = "../sand.db";
    let mut database = Database::open(db_file_path.to_string())?;
    let page = database.pager.read_page(1 as usize)?;
    println!("{:#?}", page);

    let page = database.pager.read_page(1 as usize)?;
    println!("{:#?}", page);
    let page = page.clone();

    for cell in page.cells.clone() {
        match cell {
            page::file_structures::BTreeCell::TableInteriorCell(_table_interior_cell) => todo!(),
            page::file_structures::BTreeCell::TableLeafCell(table_leaf_cell) => {
                // NOTE: This is just to see I'm reading the right things.
                match &table_leaf_cell.payload[3] {
                    page::file_structures::Value::Null => todo!(),
                    page::file_structures::Value::Integer(index) => {
                        let page = database.pager.read_page(*index as usize)?;
                        println!("{:#?}", page);
                    }
                    page::file_structures::Value::Float(_) => todo!(),
                    page::file_structures::Value::Text(_) => todo!(),
                    page::file_structures::Value::Blob(_vec) => todo!(),
                }
            }
            page::file_structures::BTreeCell::IndexInteriorCell(_index_interior_cell) => todo!(),
            page::file_structures::BTreeCell::IndexLeafCell(_index_leaf_cell) => todo!(),
        }
    }
    Ok(())
}

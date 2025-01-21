use anyhow::Result;
use file_structures::DBHeader;
use pager::Pager;
use std::{
    collections::HashMap,
    fs::File,
    sync::{Arc, RwLock},
};

pub mod errors;
pub mod file_structures;
pub mod pager;

// TODO: Check how to implement concurreny. Maybe the page cache and header and pager show be
// sharable across instances? The cache should be at least.
// Implementing Arc<RwLock<HashMap<uszie, BTreePage>>> for now. Seems like the right thing but
// still need to look moer into this.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Database {
    pub pager: Pager,
    // Should this be shared like Rc or Arc?
    header: DBHeader,
    // This I think should definitely be shared. Will look into this.
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn open(file_path: String) -> Result<Database> {
        let mut file = File::open(file_path)?;
        let page_cache = Arc::new(RwLock::new(HashMap::new()));
        let header = file_structures::read_db_header(&mut file)?;

        let pager = Pager {
            db_header: header.clone(),
            file,
            // NOTE: With Arc Clone is the way to go.
            page_cache: page_cache.clone(),
        };

        let mut tables = HashMap::new();
        tables.insert("sqlite_master".to_string(), Table::get_master_table());

        Ok(Database {
            pager,
            tables,
            header,
        })
    }
}

#[derive(Debug)]
pub struct Table {
    pub root_page: usize,
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub column_type: ColumnType,
    // TODO: Find out how to handle primary keys.
}

#[derive(Debug, PartialEq)]
pub enum ColumnType {
    // NULL value.
    NULL,
    // Signed integer stores as 0, 1, 2, 3, 4, 6, or 8 bytes depending on magnitude of value.
    Integer,
    // Floating point value stored as 8-byte IEEE floating ponit number.
    Real,
    // TEst string encoded as either UTF-8 UTF-16BE or UTF-16LE.
    Text,
    // Blob sotred exactly as input.
    Blob,
}

impl Table {
    // NOTE: Turns out `const SQLITE_MASTER = Table {}` does not work because you cannot have non
    // constant method calls in the const declaration, so we go with a function instead.
    pub fn get_master_table() -> Table {
        Table {
            root_page: 1,
            name: "sqlite_master".to_string(),
            columns: vec![
                Column {
                    name: "type".to_string(),
                    column_type: ColumnType::Text,
                },
                Column {
                    name: "name".to_string(),
                    column_type: ColumnType::Text,
                },
                Column {
                    name: "table_name".to_string(),
                    column_type: ColumnType::Text,
                },
                Column {
                    name: "root_page".to_string(),
                    column_type: ColumnType::Integer,
                },
                Column {
                    name: "sql".to_string(),
                    column_type: ColumnType::Text,
                },
            ],
        }
    }
}

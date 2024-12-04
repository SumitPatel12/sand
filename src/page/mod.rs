pub mod dbfileheader;
pub mod errors;
pub mod page;
pub mod page_parser;
mod varint;

// TODO: Verify this root page size variable. It's confusing.
#[derive(Debug, Clone)]
pub struct SQLiteSchema {
    schema_type: String,
    name: String,
    table_name: String,
    root_page: i8,
    sql: String,
}

// TODO: Decide if using this makes sense.
// enum SchemaType {
// Table = "table",
// Index = "index",
// View = "view",
// Tirgger = "trigger"
// }

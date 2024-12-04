// All of the todos are left by me, just that sometimes I forget or sometimes I address myself in
// third person, don't sweat it.
pub mod page;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    page::page_parser::parse_file()?;
    Ok(())
}

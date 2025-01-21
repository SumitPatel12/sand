use anyhow::{Ok, Result};
use std::{
    collections::HashMap,
    fs::File,
    sync::{Arc, RwLock},
    usize,
};

use crate::page::file_structures;

use super::file_structures::{BTreePage, DBHeader};

// TODO: Not sure where to put the page cache, for now keeping it both at Pager and the top level DB
// struct.
// The BTreePage shoub be wrapped in a Rc maybe? A from the user perspective the lib can be called
// and shared as a service, in that case more than one objects may reference the cached page.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Pager {
    pub db_header: DBHeader,
    pub file: File,
    pub page_cache: Arc<RwLock<HashMap<usize, Arc<BTreePage>>>>,
}

impl Pager {
    // Man, lifetimes are tough.
    // TODO: Look into why this is wrong. It's likely a lifetime issue. Maybe page cache stores a
    // reference and not the page? Since the page cache itself is Arc wrapped should the page
    // reference be wrapped in Arc as well?
    pub fn read_page(&mut self, page_index: usize) -> Result<Arc<BTreePage>> {
        println!("Pager: Trying to read page at index: {}", page_index);
        let mut page_cache = self.page_cache.write().unwrap();
        if let Some(page) = page_cache.get(&page_index) {
            println!("Got page {} from cache.", page_index);
            return Ok(page.clone());
        }

        let page = file_structures::read_page(
            &mut self.file,
            self.db_header.page_size as usize,
            page_index,
        )?;
        let arc_page = Arc::new(page);
        page_cache.insert(page_index, arc_page.clone());
        Ok(arc_page)
    }
}

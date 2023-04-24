use std::io;
use crate::filestore::FileStore;

#[derive(Debug)]
pub struct Cache {
    pub store: FileStore,
}

impl Cache {
    pub fn new(store: FileStore) -> Cache {
        Cache { store }
    }

    pub fn get_type_count(&self) -> usize {
        self.store.get_type_count()
    }

    pub fn get_file_count(&self, file_type: usize) -> io::Result<usize> {
        self.store.get_file_count(file_type)
    }
}
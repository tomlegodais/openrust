use std::io;
use bytes::Buf;
use crate::checksum_table::{ChecksumTable, Entry};
use crate::container::Container;
use crate::filestore::FileStore;
use crate::{get_crc_checksum, get_whirlpool_digest};
use crate::reference_table::ReferenceTable;

#[derive(Debug)]
pub struct Cache {
    store: FileStore,
}

impl Cache {
    pub fn new(store: FileStore) -> Cache {
        Cache { store }
    }

    pub fn create_checksum_table(&mut self) -> io::Result<ChecksumTable> {
        let size = self.get_type_count();
        let mut table = ChecksumTable::new(size as usize);

        for i in 0..size {
            let mut buf = self.store.read(255, i)?;

            let mut crc = 0;
            let mut version = 0;
            let mut whirlpool = [0; 64];

            if buf.remaining() > 0 {
                let ref_table = ReferenceTable::decode(Container::decode(&mut buf)?.data_mut())?;
                crc = get_crc_checksum(&buf);
                version = ref_table.version().unwrap_or_default();
                buf.set_position(0);
                whirlpool = get_whirlpool_digest(&buf);
            }

            table.entries_mut().insert(i, Entry::new(crc, version, whirlpool));
        }

        Ok(table)
    }

    pub fn store(&self) -> &FileStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut FileStore {
        &mut self.store
    }

    pub fn get_type_count(&self) -> usize {
        self.store.get_type_count()
    }

    pub fn get_file_count(&self, file_type: usize) -> io::Result<usize> {
        self.store.get_file_count(file_type)
    }
}
use std::collections::HashMap;

#[derive(Debug)]
pub struct ChecksumTable {
    entries: HashMap<usize, Entry>,
}

impl ChecksumTable {
    pub fn new(size: usize) -> Self {
        ChecksumTable {
            entries: HashMap::with_capacity(size),
        }
    }

    pub fn get_entry(&self, index: usize) -> Option<&Entry> {
        self.entries.get(&index)
    }

    pub fn entries(&self) -> &HashMap<usize, Entry> {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut HashMap<usize, Entry> {
        &mut self.entries
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    crc: u32,
    version: i32,
    whirlpool: [u8; 64],
}

impl Entry {
    pub fn new(crc: u32, version: i32, whirlpool: [u8; 64]) -> Self {
        Entry {
            crc,
            version,
            whirlpool,
        }
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn set_crc(&mut self, crc: u32) {
        self.crc = crc;
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn set_version(&mut self, version: i32) {
        self.version = version;
    }

    pub fn whirlpool(&self) -> &[u8; 64] {
        &self.whirlpool
    }

    pub fn set_whirlpool(&mut self, whirlpool: [u8; 64]) {
        self.whirlpool = whirlpool;
    }
}
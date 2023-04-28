use std::io::{self, Cursor, Write};
use byteorder::{BigEndian, WriteBytesExt};
use num_bigint::BigUint;
use crate::{encrypt_rsa, hash_whirlpool};

#[derive(Debug)]
pub struct ChecksumTable {
    entries: Vec<Entry>,
}

impl ChecksumTable {
    pub fn new(size: usize) -> Self {
        ChecksumTable {
            entries: Vec::with_capacity(size),
        }
    }

    pub fn encode(&self) -> io::Result<Cursor<Vec<u8>>> {
        self.encode_impl(false, None, None)
    }

    pub fn encode_impl(&self, whirlpool: bool, modulus: Option<BigUint>, private_key: Option<BigUint>) -> io::Result<Cursor<Vec<u8>>> {
        let mut buf = Vec::new();
        if whirlpool {
            buf.write_u8(self.entries.len() as u8)?;
        }

        for entry in &self.entries {
            buf.write_u32::<BigEndian>(entry.crc)?;
            buf.write_u32::<BigEndian>(entry.version as u32)?;
            if whirlpool {
                buf.write_all(entry.whirlpool())?;
            }
        }

        let bytes = buf.clone();
        if whirlpool {
            let mut temp = Vec::with_capacity(65);
            temp.push(0);
            temp.extend_from_slice(&hash_whirlpool(&bytes));
            let mut temp = temp.into_boxed_slice();

            if let (Some(modulus), Some(private_key)) = (modulus, private_key) {
                temp = encrypt_rsa(&temp, modulus, private_key).into_boxed_slice();
            }

            buf.extend_from_slice(&temp);
        }

        Ok(Cursor::new(buf))
    }

    pub fn get_entry(&self, index: usize) -> Option<&Entry> {
        self.entries.get(index)
    }

    pub fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut Vec<Entry> {
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
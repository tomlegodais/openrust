use std::io::{self, Cursor};
use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug)]
pub struct Index {
    size: u32,
    sector: u32,
}

impl Index {
    pub const SIZE: usize = 6;

    pub fn decode(buf: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let size = buf.read_u24::<BigEndian>()?;
        let sector = buf.read_u24::<BigEndian>()?;

        Ok(Self { size, sector })
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn sector(&self) -> u32 {
        self.sector
    }
}
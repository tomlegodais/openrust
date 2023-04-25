use bytes::Buf;
use std::io::{self, Cursor, Error, ErrorKind, prelude::*};
use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug)]
pub struct Sector {
    type_id: u8,
    id: u16,
    chunk: u16,
    next_sector: u32,
    data: [u8; Sector::DATA_SIZE],
}

impl Sector {
    pub const HEADER_SIZE: usize = 8;
    pub const DATA_SIZE: usize = 512;
    pub const SIZE: usize = Self::HEADER_SIZE + Self::DATA_SIZE;

    pub fn decode(buf: &mut Cursor<&Vec<u8>>) -> io::Result<Self> {
        if buf.remaining() != Self::SIZE {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid buffer size"));
        }

        let id = buf.read_u16::<BigEndian>()?;
        let chunk = buf.read_u16::<BigEndian>()?;
        let next_sector = buf.read_u24::<BigEndian>()? & 0x00ffffff;
        let type_id = buf.read_u8()?;
        let mut data = [0u8; Self::DATA_SIZE];
        buf.read_exact(&mut data)?;

        Ok(Self { type_id, id, chunk, next_sector, data })
    }

    pub fn type_id(&self) -> u8 {
        self.type_id
    }
    
    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn chunk(&self) -> u16 {
        self.chunk
    }

    pub fn next_sector(&self) -> u32 {
        self.next_sector
    }

    pub fn data(&self) -> &[u8; Sector::DATA_SIZE] {
        &self.data
    }
}
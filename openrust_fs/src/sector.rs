use bytes::Buf;
use std::io::{self, Cursor, Error, ErrorKind, prelude::*};
use byteorder::{BigEndian, ReadBytesExt};


pub struct Sector {
    pub type_id: u8,
    pub id: u16,
    pub chunk: u16,
    pub next_sector: u32,
    pub data: [u8; Sector::DATA_SIZE],
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
}
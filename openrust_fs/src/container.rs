use std::io::{self, Cursor, Error, ErrorKind, Read};
use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::{bunzip2, bzip2, decipher_xtea, gunzip, gzip};

static NULL_KEY: [i32; 4] = [0; 4];

const DATA_OFFSET: usize = 5;
pub const COMPRESSION_NONE: u8 = 0;
const COMPRESSION_BZIP2: u8 = 1;
const COMPRESSION_GZIP: u8 = 2;

#[derive(Debug)]
pub struct Container {
    type_id: u8,
    data: Cursor<Vec<u8>>,
    version: i16,
}

impl Container {
    pub fn new(type_id: u8, data: Cursor<Vec<u8>>) -> Self {
        Self { type_id, data, version: -1 }
    }

    pub fn decode(buffer: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Self::decode_with_key(buffer, &NULL_KEY)
    }

    pub fn decode_with_key(mut buffer: &mut Cursor<Vec<u8>>, key: &[i32; 4]) -> io::Result<Self> {
        let type_id = buffer.read_u8()?;
        let length = buffer.read_i32::<BigEndian>()? as usize;

        if *key != [0i32, 0i32, 0i32, 0i32] {
            let data_len = length + if type_id == COMPRESSION_NONE { DATA_OFFSET } else { DATA_OFFSET + 4 };

            decipher_xtea(&mut buffer, DATA_OFFSET, data_len, key)?;
        }

        if type_id == COMPRESSION_NONE {
            let data = Cursor::new(buffer.clone()
                .into_inner()
                .split_off(DATA_OFFSET));
            let version = Self::decode_version(buffer)?;

            Ok(Self { type_id, data, version })
        } else {
            let uncompressed_length = buffer.get_i32() as usize;
            let mut compressed_buf = vec![0; length];
            buffer.read_exact(&mut compressed_buf)?;

            let compressed_buf = Bytes::from(compressed_buf);
            let decompress_fn: fn(&[u8]) -> io::Result<Vec<u8>> = match type_id {
                COMPRESSION_BZIP2 => bunzip2,
                COMPRESSION_GZIP => gunzip,
                _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid compression type"))
            };

            let uncompressed = decompress_fn(&compressed_buf)?;
            if uncompressed.len() != uncompressed_length {
                return Err(Error::new(ErrorKind::InvalidData, "Length mismatch"));
            }

            let version = Self::decode_version(buffer)?;
            Ok(Self { type_id, data: Cursor::new(uncompressed), version })
        }
    }

    pub fn encode(self) -> io::Result<Cursor<Vec<u8>>> {
        let remaining = self.data.remaining();
        let mut bytes = BytesMut::with_capacity(remaining);
        bytes.put(self.data);

        let compressed = match self.type_id {
            COMPRESSION_NONE => bytes.to_vec(),
            COMPRESSION_BZIP2 => bzip2(bytes.as_ref())?,
            COMPRESSION_GZIP => gzip(bytes.as_ref())?,
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid compression type")),
        };

        let header = DATA_OFFSET + (if self.type_id == COMPRESSION_NONE { 0 } else { 4 }) + (if self.version != -1 { 2 } else { 0 });
        let mut buf = BytesMut::with_capacity(header + compressed.len());

        buf.put_u8(self.type_id);
        buf.put_u32(compressed.len() as u32);

        if self.type_id != COMPRESSION_NONE {
            buf.put_u32(remaining as u32);
        }

        buf.put_slice(&compressed);

        if self.version != -1 {
            buf.put_u16(self.version as u16);
        }

        Ok(Cursor::new(buf.to_vec()))
    }

    fn decode_version(cursor: &mut Cursor<Vec<u8>>) -> io::Result<i16> {
        if cursor.remaining() >= 2 { cursor.read_i16::<BigEndian>() } else { Ok(-1) }
    }

    pub fn type_id(&self) -> u8 {
        self.type_id
    }

    pub fn set_type_id(&mut self, type_id: u8) {
        self.type_id = type_id;
    }

    pub fn data(&self) -> &Cursor<Vec<u8>> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Cursor<Vec<u8>> {
        &mut self.data
    }

    pub fn version(&self) -> i16 {
        self.version
    }

    pub fn set_version(&mut self, version: i16) {
        self.version = version;
    }
}
use std::io::{self, Cursor, Error, ErrorKind, Read};
use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, Bytes};
use crate::{bunzip2, decipher_xtea, gunzip};

static NULL_KEY: [i32; 4] = [0; 4];

const DATA_OFFSET: usize = 5;
const COMPRESSION_NONE: u8 = 0;
const COMPRESSION_BZIP2: u8 = 1;
const COMPRESSION_GZIP: u8 = 2;

#[derive(Debug)]
pub struct Container {
    pub type_id: u8,
    pub data: Cursor<Vec<u8>>,
    pub version: i16,
}

impl Container {
    pub fn decode(buffer: Cursor<Vec<u8>>) -> io::Result<Self> {
        Self::decode_with_key(buffer, &NULL_KEY)
    }

    pub fn decode_with_key(mut buffer: Cursor<Vec<u8>>, key: &[i32; 4]) -> io::Result<Self> {
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
            let version = decode_version(buffer)?;

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

            let version = decode_version(buffer)?;
            Ok(Self { type_id, data: Cursor::new(uncompressed), version })
        }
    }
}

fn decode_version(mut cursor: Cursor<Vec<u8>>) -> io::Result<i16> {
    Ok(if cursor.remaining() >= 2 { cursor.read_i16::<BigEndian>()? } else { -1 })
}
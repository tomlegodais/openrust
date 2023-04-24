use std::io::{self, Cursor, Error, ErrorKind, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;

pub mod cache;
pub mod container;
pub mod filestore;
mod index;
mod sector;

const GOLDEN_RATIO: u32 = 0x9E3779B9;
const ROUNDS: u32 = 32;

fn decipher_xtea(buffer: &mut Cursor<Vec<u8>>, offset: usize, length: usize, key: &[i32; 4]) -> io::Result<()> {
    let initial_position = buffer.position();
    let num_quads = (length - offset) / 8;

    for i in 0..num_quads {
        let mut sum = GOLDEN_RATIO.wrapping_mul(ROUNDS);
        let index = offset + i * 8;
        buffer.set_position(index as u64);

        let mut v0 = buffer.read_u32::<BigEndian>()?;
        let mut v1 = buffer.read_u32::<BigEndian>()?;

        for _ in 0..ROUNDS {
            v1 = v1.wrapping_sub((((v0 << 4) ^ (v0 >> 5)).wrapping_add(v0)) ^ (sum.wrapping_add(key[((sum >> 11) & 3) as usize] as u32)));
            sum = sum.wrapping_sub(GOLDEN_RATIO);
            v0 = v0.wrapping_sub((((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1)) ^ (sum.wrapping_add(key[(sum & 3) as usize] as u32)));
        }

        buffer.set_position(index as u64);
        buffer.write_u32::<BigEndian>(v0)?;
        buffer.write_u32::<BigEndian>(v1)?;
    }

    buffer.set_position(initial_position);
    Ok(())
}

fn bunzip2(compressed: &[u8]) -> io::Result<Vec<u8>> {
    let mut bzip2 = Vec::with_capacity(compressed.len() + 4);
    bzip2.write_all(b"BZh1")?;
    bzip2.extend_from_slice(compressed);

    let mut decoder = BzDecoder::new(Cursor::new(bzip2));
    let mut uncompressed = Vec::new();
    match decoder.read_to_end(&mut uncompressed) {
        Ok(_) => Ok(uncompressed),
        Err(e) => Err(Error::new(ErrorKind::Other, e))
    }
}

fn gunzip(compressed: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(compressed);
    let mut uncompressed = Vec::new();
    match decoder.read_to_end(&mut uncompressed) {
        Ok(_) => Ok(uncompressed),
        Err(e) => Err(Error::new(ErrorKind::Other, e))
    }
}
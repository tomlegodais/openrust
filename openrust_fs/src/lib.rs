use std::io::{self, Cursor, Error, ErrorKind, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bzip2::read::{BzDecoder};
use bzip2::write::BzEncoder;
use crc32fast::Hasher;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use num_bigint::BigUint;
use whirlpool::{Whirlpool, Digest};
use whirlpool::digest::FixedOutput;

pub mod filestore;
pub mod cache;
pub mod container;
pub mod reference_table;
pub mod checksum_table;
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

fn bzip2(uncompressed: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = BzEncoder::new(Vec::new(), bzip2::Compression::default());
    encoder.write_all(uncompressed)?;
    encoder.finish()
}

fn gunzip(compressed: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(compressed);
    let mut uncompressed = Vec::new();
    match decoder.read_to_end(&mut uncompressed) {
        Ok(_) => Ok(uncompressed),
        Err(e) => Err(Error::new(ErrorKind::Other, e))
    }
}

fn gzip(uncompressed: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(uncompressed)?;
    encoder.finish()
}

fn get_crc_checksum(buf: &Cursor<Vec<u8>>) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(&buf.get_ref());
    hasher.finalize()
}

fn get_whirlpool_digest(buf: &Cursor<Vec<u8>>) -> [u8; 64] {
    hash_whirlpool(buf.get_ref())
}

fn hash_whirlpool(bytes: &Vec<u8>) -> [u8; 64] {
    let mut whirlpool = Whirlpool::new();
    whirlpool.update(&bytes);
    let result: [u8; 64] = whirlpool.finalize_fixed().into();
    result
}

fn encrypt_rsa(data: &[u8], modulus: BigUint, private_key: BigUint) -> Vec<u8> {
    let data = BigUint::from_bytes_be(data);
    let ciphertext = data.modpow(&private_key, &modulus);
    ciphertext.to_bytes_be()
}
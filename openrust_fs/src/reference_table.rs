use std::collections::HashMap;
use std::io::{self, Cursor, Read};
use byteorder::{BigEndian, ReadBytesExt};

const FLAG_IDENTIFIERS: u8 = 0x01;
const FLAG_WHIRLPOOL: u8 = 0x02;

#[derive(Debug)]
struct ChildEntry {
    identifier: Option<i32>,
}

#[derive(Debug)]
struct Entry {
    identifier: Option<i32>,
    crc: i32,
    whirlpool: [u8; 64],
    version: i32,
    entries: HashMap<i32, ChildEntry>,
}

#[derive(Debug)]
pub struct ReferenceTable {
    format: u8,
    version: Option<i32>,
    flags: u8,
    entries: HashMap<i32, Entry>,
}

impl ReferenceTable {
    pub fn decode(mut buffer: Cursor<Vec<u8>>) -> io::Result<Self> {
        let mut table = ReferenceTable {
            format: 0,
            version: None,
            flags: 0,
            entries: HashMap::new(),
        };

        table.format = buffer.read_u8()?;
        if table.format >= 6 {
            table.version = Some(buffer.read_i32::<BigEndian>()?);
        }
        table.flags = buffer.read_u8()?;

        let ids_length = buffer.read_u16::<BigEndian>()? as usize;
        let mut ids = vec![0; ids_length];
        let mut accumulator = 0;
        let mut size = -1;

        for i in 0..ids.len() {
            let delta = buffer.read_i16::<BigEndian>()?;
            ids[i] = accumulator + i32::from(delta);
            accumulator = ids[i];
            if ids[i] > size {
                size = ids[i];
            }
        }
        size += 1;

        for id in &ids {
            table.entries.insert(*id, Entry {
                identifier: None,
                crc: 0,
                whirlpool: [0; 64],
                version: 0,
                entries: HashMap::new(),
            });
        }

        if table.flags & FLAG_IDENTIFIERS != 0 {
            for id in &ids {
                let entry = table.entries.get_mut(id).unwrap();
                entry.identifier = Some(buffer.read_i32::<BigEndian>()?);
            }
        }

        for id in &ids {
            let entry = table.entries.get_mut(id).unwrap();
            entry.crc = buffer.read_i32::<BigEndian>()?;
        }

        if table.flags & FLAG_WHIRLPOOL != 0 {
            for id in &ids {
                buffer.read_exact(&mut table.entries.get_mut(id).unwrap().whirlpool)?;
            }
        }

        for id in &ids {
            let entry = table.entries.get_mut(id).unwrap();
            entry.version = buffer.read_i32::<BigEndian>()?;
        }

        let mut members = vec![vec![]; size as usize];
        for id in &ids {
            let child_size = buffer.read_u16::<BigEndian>()?;
            members[*id as usize] = vec![0; child_size as usize];
        }

        for id in &ids {
            accumulator = 0;
            size = -1;

            for i in 0..members[*id as usize].len() {
                let delta = buffer.read_i16::<BigEndian>()?;
                members[*id as usize][i] = accumulator + i32::from(delta);
                accumulator = members[*id as usize][i];
                if members[*id as usize][i] > size {
                    size = members[*id as usize][i];
                }
            }

            for child in &members[*id as usize] {
                let entry = table.entries.get_mut(id).unwrap();
                entry.entries.insert(*child, ChildEntry { identifier: None });
            }
        }

        if table.flags & FLAG_IDENTIFIERS != 0 {
            for id in &ids {
                for child in &members[*id as usize] {
                    let entry = table.entries.get_mut(id).unwrap().entries.get_mut(child).unwrap();
                    entry.identifier = Some(buffer.read_i32::<BigEndian>()?);
                }
            }
        }

        Ok(table)
    }
}
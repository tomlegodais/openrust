use std::fs::File;
use std::io::{self, Cursor, Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;
use crate::index::Index;
use crate::sector::Sector;

#[derive(Debug)]
pub struct FileStore {
    data_channel: File,
    index_channels: Vec<File>,
    meta_channel: File,
}

impl FileStore {
    const MAIN_FILE_CACHE_DATA: &'static str = "main_file_cache.dat2";
    const MAIN_FILE_CACHE_META: &'static str = "main_file_cache.idx255";
    const MAIN_FILE_CACHE_INDEX_PREFIX: &'static str = "main_file_cache.idx";

    pub fn open<P: AsRef<Path>>(root: P) -> io::Result<Self> {
        let root = root.as_ref();
        let data_file = File::open(root.join(Self::MAIN_FILE_CACHE_DATA))?;
        let meta_file = File::open(root.join(Self::MAIN_FILE_CACHE_META))?;
        let index_files = (0..=255)
            .map(|i| root.join(format!("{}{}", Self::MAIN_FILE_CACHE_INDEX_PREFIX, i)))
            .take_while(|p| p.exists())
            .map(File::open)
            .collect::<io::Result<Vec<_>>>()?;

        if index_files.is_empty() {
            return Err(Error::new(ErrorKind::NotFound, "No index files found"));
        }

        Ok(FileStore { data_channel: data_file, index_channels: index_files, meta_channel: meta_file })
    }

    pub fn read(&mut self, type_id: usize, file_id: usize) -> io::Result<Cursor<Vec<u8>>> {
        if type_id >= self.index_channels.len() && type_id != 255 {
            let message = format!("Index channel not found for type ID: {}", type_id);
            return Err(Error::new(ErrorKind::NotFound, message));
        }

        let index_channel = if type_id == 255 {
            &mut self.meta_channel
        } else {
            &mut self.index_channels[type_id]
        };

        let ptr = (file_id * Index::SIZE) as u64;
        if ptr >= index_channel.metadata()?.len() {
            let message = format!("Index entry not found for file ID: {}", file_id);
            return Err(Error::new(ErrorKind::NotFound, message));
        }

        let mut buf = vec![0; Index::SIZE];
        index_channel.seek(SeekFrom::Start(ptr))?;
        index_channel.read_exact(&mut buf)?;

        let index = Index::decode(&mut Cursor::new(buf))?;

        let mut data = vec![0; index.size as usize];
        let mut buf = vec![0; Sector::SIZE];

        let mut ptr = (index.sector as usize * Sector::SIZE) as u64;
        let mut remaining = index.size;

        while remaining > 0 {
            self.data_channel.seek(SeekFrom::Start(ptr))?;
            self.data_channel.read_exact(&mut buf)?;

            let sector = Sector::decode(&mut Cursor::new(&buf))?;

            if remaining >= Sector::DATA_SIZE as u32 {
                data.splice(
                    (sector.chunk as usize * Sector::DATA_SIZE)..((sector.chunk as usize + 1) * Sector::DATA_SIZE),
                    sector.data.iter().cloned(),
                );

                ptr = sector.next_sector as u64 * Sector::SIZE as u64;
                remaining -= Sector::DATA_SIZE as u32;
            } else {
                data.splice(
                    (sector.chunk as usize * Sector::DATA_SIZE)..,
                    sector.data[0..remaining as usize].iter().cloned(),
                );

                remaining = 0;
            }
        }

        return Ok(Cursor::new(data));
    }

    pub fn get_type_count(&self) -> usize {
        self.index_channels.len()
    }

    pub fn get_file_count(&self, file_type: usize) -> io::Result<usize> {
        if file_type >= self.index_channels.len() && file_type != 255 {
            return Err(Error::new(ErrorKind::NotFound, "Invalid file type"));
        }

        if file_type == 255 {
            let meta_size = self.meta_channel.metadata()?.len();
            return Ok((meta_size / Index::SIZE as u64) as usize);
        }

        let index_size = self.index_channels[file_type].metadata()?.len();
        Ok((index_size / Index::SIZE as u64) as usize)
    }
}

impl Drop for FileStore {
    fn drop(&mut self) {
        let _ = self.data_channel.flush();
        let _ = self.meta_channel.flush();

        for channel in &mut self.index_channels {
            let _ = channel.flush();
        }
    }
}
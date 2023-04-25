use openrust_fs::cache::Cache;
use openrust_fs::filestore::FileStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cache = Cache::new(FileStore::open("openrust_data/fs")?);
    let checksum_table = cache.create_checksum_table()?;

    let mut crc = [0u32; 28];
    for i in 0..crc.len() {
        crc[i] = checksum_table
            .get_entry(i)
            .map(|e| e.crc())
            .unwrap_or(0);
    }

    println!("{:#?}", crc);

    return Ok(());
}

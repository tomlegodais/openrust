use openrust_fs::cache::Cache;
use openrust_fs::container::Container;
use openrust_fs::filestore::FileStore;
use openrust_fs::reference_table::ReferenceTable;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cache = Cache::new(FileStore::open("openrust_data/fs")?);
    let container = Container::decode(cache.store.read(255, 19)?)?;
    let reference_table = ReferenceTable::decode(container.data)?;

    println!("{:#?}", reference_table);

    return Ok(());
}

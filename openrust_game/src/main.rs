use openrust_fs::cache::Cache;
use openrust_fs::container::Container;
use openrust_fs::filestore::FileStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let key = [-895585464, -461480444, -1851545592, -837241102];
    // let id = 2033;

    let mut cache = Cache::new(FileStore::open("openrust_data/fs")?);
    let buffer = cache.store.read(255, 3)?;
    let buffer = Container::decode(buffer)?;

    println!("{:?}", buffer);

    return Ok(());
}

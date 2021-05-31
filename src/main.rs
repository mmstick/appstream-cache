use std::path::Path;
use appstream_cache::Database;

pub fn main() {
    smol::block_on(async move {
        println!("refreshing appstream components");

        let db = Database::new(Path::new("testing.db"));

        db.refresh_appstream_components().await;

        for package in db.search_for("firefox").await {
            println!("package: {:#?}", package);
        }
    })
}
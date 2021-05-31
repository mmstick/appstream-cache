#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde;

pub mod appstream;
pub mod dep11;
pub mod yaml;

pub use self::appstream::AppstreamPackage;

use std::path::Path;

pub enum PackageEvent {
    AppStream {
        origin: String,
        info: AppstreamPackage,
    },

    MediaUrl {
        origin: String,
        base_url: String,
    }
}


pub struct Database {
    db: sled::Db,
    apps: sled::Tree,
    origins: sled::Tree,
}

impl Database {
    pub fn new(path: &Path) -> Self {
        let db = sled::open(path).unwrap();
        let apps = db.open_tree("apps").unwrap();
        let origins = db.open_tree("origins").unwrap();

        Self { db, apps, origins }
    }

    pub async fn refresh_appstream_components(&self) -> anyhow::Result<()> {
        let _ = self.apps.clear();
        let _ = self.origins.clear();

        // Each package list is going to contain a stream of packages we'll collate
        let (tx, rx) = smol::channel::unbounded();

        // This executor shall spawn an I/O task for each package list, and then all information
        // will be converged into a singular location in our sled database. The purpose of doing
        // so is to make future queries for packages quick and efficient.
        let executor = &smol::LocalExecutor::new();

        executor.run(async move {
            dep11::fetch(executor, tx)?;

            while let Ok(event) = rx.recv().await {
                match event {
                    PackageEvent::AppStream { origin, mut info } => {
                        println!("Serializing {} from {}", info.id, origin);

                        info.origin = Some(origin);

                        if let Ok(serialized) = bincode::serialize(&info) {
                            let _ = self.apps.insert(info.id.as_bytes(), serialized);
                        }
                    }

                    PackageEvent::MediaUrl { origin, base_url } => {
                        let _ = self.origins.insert(origin.as_bytes(), base_url.as_bytes());
                    }
                }
            }

            println!("flushing");
            let _ = self.db.flush_async().await;

            Ok(())
        }).await
    }

    pub async fn search_for(&self, package: &str) -> Vec<AppstreamPackage> {
        let mut packages = Vec::new();

        for result in self.apps.iter() {
            if let Ok((key, value)) = result {
                if let (Ok(key), Ok(value)) = (std::str::from_utf8(&key), bincode::deserialize::<AppstreamPackage>(&value)) {
                    if key.contains(package) {
                        packages.push(value);
                    }
                }
            }
        }

        packages
    }
}




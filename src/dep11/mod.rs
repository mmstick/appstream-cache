pub mod codec;

use anyhow::Context;
use crate::PackageEvent;
use os_str_bytes::OsStrBytes;
use self::codec::Dep11Splitter;
use smol::channel::Sender;
use std::fs;
use std::path::{Path, PathBuf};

const LISTS: &str = "/var/lib/apt/lists";

pub fn fetch<'a>(executor: &smol::LocalExecutor<'a>, tx: Sender<PackageEvent>) -> anyhow::Result<()> {
    let lists = Path::new(LISTS);

    if !lists.exists() {
        return Ok(());
    }

    // Fetches all DEP11 package lists in the system
    let dep11_entries = fs::read_dir(lists)
        .context("failed to read apt lists dir")?
        .filter_map(Result::ok)
        .filter(|entry| contains_slice(&entry.file_name().to_raw_bytes(), b"dep11_Components"));

    for package_entry in dep11_entries {
        executor.spawn(read_dep11_components(package_entry.path(), tx.clone())).detach();
    }

    Ok(())
}

async fn read_dep11_components(path: PathBuf, tx: Sender<PackageEvent>) -> anyhow::Result<()> {
    use flate2::read::GzDecoder;
    use futures_codec::FramedRead;
    use futures_lite::prelude::*;
    use std::fs::File;

    println!("reading {:?}", path);

    let decoder = File::open(&path)
        .map(GzDecoder::new)
        .expect("failed to open file");

    let decoder = smol::Unblock::new(decoder);

    let mut stream = FramedRead::new(decoder, Dep11Splitter::default());

    if let Some(result) = stream.next().await {
        let info = result.unwrap();
        if let Some(header) = stream.decoder_mut().header.as_ref() {
            let origin = header.origin.clone();

            if let Some(base_url) = header.media_base_url.clone() {
                tx.send(PackageEvent::MediaUrl { origin: origin.clone(), base_url }).await;
            }

            tx.send(PackageEvent::AppStream { origin: origin.clone(), info }).await;

            while let Some(event) = stream.next().await {
                if let Ok(info) = event {
                    tx.send(PackageEvent::AppStream { origin: origin.clone(), info }).await;
                }
            }
        }
    }

    println!("exiting from {:?}", path);

    Ok(())
}

fn contains_slice(slice: &[u8], pattern: &[u8]) -> bool {
    slice.windows(pattern.len()).any(|v| v == pattern)
}

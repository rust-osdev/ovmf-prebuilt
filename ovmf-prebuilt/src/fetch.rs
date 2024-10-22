use crate::{Error, Source};
use log::info;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};
use tar::Archive;
use ureq::Agent;

/// User-Agent header to send with download requests.
const USER_AGENT: &str = "https://github.com/rust-osdev/ovmf-prebuilt";

/// Maximum number of bytes to download (10 MiB).
const MAX_DOWNLOAD_SIZE_IN_BYTES: usize = 10 * 1024 * 1024;

/// Update the local cache. Does nothing if the cache is already up to date.
pub(crate) fn update_cache(source: Source, prebuilt_dir: &Path) -> Result<(), Error> {
    let hash_path = prebuilt_dir.join("sha256");

    // Check if the hash file already has the expected hash in it. If so, assume
    // that we've already got the correct prebuilt downloaded and unpacked.
    if let Ok(current_hash) = fs::read_to_string(&hash_path) {
        if current_hash == source.sha256 {
            return Ok(());
        }
    }

    let base_url = "https://github.com/rust-osdev/ovmf-prebuilt/releases/download";
    let url = format!(
        "{base_url}/{release}/{release}-bin.tar.xz",
        release = source.tag
    );

    let data = download_url(&url)?;

    // Validate the hash.
    let actual_hash = format!("{:x}", Sha256::digest(&data));
    if actual_hash != source.sha256 {
        return Err(Error::HashMismatch {
            actual: actual_hash,
            expected: source.sha256.to_owned(),
        });
    }

    // Unpack the tarball.
    let decompressed = decompress(&data)?;

    // Clear out the existing prebuilt dir, if present.
    let _ = fs::remove_dir_all(prebuilt_dir);

    // Extract the files.
    extract(&decompressed, prebuilt_dir).map_err(Error::Extract)?;

    // Write out the hash file. When we upgrade to a new release of
    // ovmf-prebuilt, the hash will no longer match, triggering a fresh
    // download.
    fs::write(&hash_path, actual_hash).map_err(Error::HashWrite)?;

    Ok(())
}

/// Download `url` and return the raw data.
fn download_url(url: &str) -> Result<Vec<u8>, Error> {
    let agent: Agent = ureq::AgentBuilder::new().user_agent(USER_AGENT).build();

    // Download the file.
    info!("downloading {url}");
    let resp = agent
        .get(url)
        .call()
        .map_err(|err| Error::Request(Box::new(err)))?;
    let mut data = Vec::with_capacity(MAX_DOWNLOAD_SIZE_IN_BYTES);
    resp.into_reader()
        // Limit the size of the download.
        .take(MAX_DOWNLOAD_SIZE_IN_BYTES.try_into().unwrap())
        .read_to_end(&mut data)
        .map_err(Error::Download)?;
    info!("received {} bytes", data.len());

    Ok(data)
}

fn decompress(data: &[u8]) -> Result<Vec<u8>, Error> {
    info!("decompressing tarball");
    let mut decompressed = Vec::new();
    let mut compressed = Cursor::new(data);
    lzma_rs::xz_decompress(&mut compressed, &mut decompressed).map_err(Error::Decompress)?;
    Ok(decompressed)
}

/// Extract the tarball's files into `prebuilt_dir`.
///
/// `tarball_data` is raw decompressed tar data.
fn extract(tarball_data: &[u8], prebuilt_dir: &Path) -> Result<(), io::Error> {
    let cursor = Cursor::new(tarball_data);
    let mut archive = Archive::new(cursor);

    // Extract each file entry.
    for entry in archive.entries()? {
        let mut entry = entry?;

        // Skip directories.
        if entry.size() == 0 {
            continue;
        }

        let path = entry.path()?;
        // Strip the leading directory, which is the release name.
        let path: PathBuf = path.components().skip(1).collect();

        let dir = path.parent().unwrap();
        let dst_dir = prebuilt_dir.join(dir);
        let dst_path = prebuilt_dir.join(path);
        info!("unpacking to {}", dst_path.display());
        fs::create_dir_all(dst_dir)?;
        entry.unpack(dst_path)?;
    }

    Ok(())
}

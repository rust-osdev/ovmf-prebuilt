use std::fmt::{self, Display, Formatter};
use std::io;

/// Cache or fetch error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Hash of the downloaded file does not match the expected value.
    HashMismatch {
        /// Expected hash.
        expected: String,
        /// Actual hash.
        actual: String,
    },

    /// Failed to write the hash file.
    HashWrite(io::Error),

    /// Remote request failed.
    Request(Box<ureq::Error>),

    /// Download failed.
    Download(io::Error),

    /// Tarball decompression failed.
    Decompress(lzma_rs::error::Error),

    /// Tarball extraction failed.
    Extract(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::HashMismatch { expected, actual } => write!(
                f,
                "file hash {actual} does not match expected hash {expected}"
            ),
            // `source` returns non-None for these variants, so do not
            // format the inner error.
            Self::HashWrite(_) => write!(f, "failed to write hash file"),
            Self::Request(_) => write!(f, "remote request failed"),
            Self::Download(_) => write!(f, "download failed"),
            Self::Decompress(_) => write!(f, "tarball decompression failed"),
            Self::Extract(_) => write!(f, "tarball extraction failed"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::HashMismatch { .. } => None,
            Self::HashWrite(err) => Some(err),
            Self::Request(err) => Some(err),
            Self::Download(err) => Some(err),
            Self::Extract(err) => Some(err),
            Self::Decompress(err) => Some(err),
        }
    }
}

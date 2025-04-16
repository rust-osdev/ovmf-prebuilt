//! Download, cache, and access [OVMF] prebuilts.
//!
//! [OVMF]: https://github.com/tianocore/edk2/tree/master/OvmfPkg#readme
//!
//! # Example
//!
//! ```
//! use ovmf_prebuilt::{Arch, FileType, Source, Prebuilt};
//! use std::path::Path;
//!
//! let prebuilt = Prebuilt::fetch(Source::LATEST, "target/ovmf")
//!     .expect("failed to update prebuilt");
//! assert_eq!(
//!     prebuilt.get_file(Arch::X64, FileType::Code),
//!     Path::new("target/ovmf/x64/code.fd")
//! );
//! ```

#![warn(missing_docs)]

mod error;
mod fetch;

use fetch::update_cache;
use std::path::{Path, PathBuf};

pub use error::Error;

/// Which prebuilt to download.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source {
    /// Release tag name, e.g. "edk2-stable202408-r1".
    pub tag: &'static str,

    /// SHA-256 hash of the compressed tarball.
    pub sha256: &'static str,
}

#[allow(missing_docs)]
impl Source {
    pub const EDK2_STABLE202408_R1: Self = Self {
        tag: "edk2-stable202408-r1",
        sha256: "63a9217ddd51fa45d0a89fd83c483cc971765de6bb08e83cf70836b0baff0d48",
    };

    pub const EDK2_STABLE202408_01_R1: Self = Self {
        tag: "edk2-stable202408.01-r1",
        sha256: "1b4c7d7603517482a3c4461ba43044c4c7e0a7930274d77eb19600b7dcd9b838",
    };

    pub const EDK2_STABLE202411_R1: Self = Self {
        tag: "edk2-stable202411-r1",
        sha256: "963fc6cef6a0560cec97381ed22a7d5c76f440c8212529a034cb465466cd57cc",
    };

    pub const EDK2_STABLE202502_R1: Self = Self {
        tag: "edk2-stable202502-r1",
        sha256: "6d6122e88cdc09e1ffafb6a39fbdbfba668a6ded3f2a032b2cd6c0b7ff6d69df",
    };

    pub const EDK2_STABLE202502_R2: Self = Self {
        tag: "edk2-stable202502-r2",
        sha256: "dd59d3d52f0a643f07a488f80ab40f89c30f360999d98cdffb30e1eba5476641",
    };

    /// Latest release tag.
    ///
    /// Note that this is not necessarily the latest prebuilt available
    /// from the git repo.
    pub const LATEST: Self = Self::EDK2_STABLE202502_R2;
}

/// UEFI architecture.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Arch {
    Aarch64,
    Ia32,
    Riscv64,
    X64,
}

impl Arch {
    /// Convert to a string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Aarch64 => "aarch64",
            Self::Ia32 => "ia32",
            Self::Riscv64 => "riscv64",
            Self::X64 => "x64",
        }
    }
}

/// Type of file within the prebuilt archive.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(missing_docs)]
pub enum FileType {
    Code,
    Vars,
    Shell,
}

impl FileType {
    /// Convert to a string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Code => "code.fd",
            Self::Vars => "vars.fd",
            Self::Shell => "shell.efi",
        }
    }
}

/// Cached prebuilt.
pub struct Prebuilt {
    dir: PathBuf,
}

impl Prebuilt {
    /// Fetch a prebuilt from a local cache. If the cache is out of
    /// date, the prebuilt is downloaded and the cache is updated.
    ///
    /// The SHA-256 hash of the original prebuilt is stored in
    /// `<prebuilt_dir>/sha256`. This is used to determine whether the
    /// cache is up-to-date. Note that if some external process modifies
    /// the cached files but leaves the `sha256` file unmodified, this
    /// code will not detect that the cache is invalid.
    ///
    /// If the cache is updated, the downloaded prebuilt's hash will be
    /// checked against [`source.sha256`]. An error will be
    /// returned if the hash does not match, and the filesystem will not
    /// be modified. This ensures that if you pin this library in
    /// `Cargo.lock`, and use one of the [`Source`] associated
    /// constants, the library will never unpack unverified files. This
    /// provides some protection against a malicious attack modifying
    /// the release tarballs on Github.
    ///
    /// [`source.sha256`]: Source::sha256
    pub fn fetch<P: AsRef<Path>>(source: Source, prebuilt_dir: P) -> Result<Self, Error> {
        let prebuilt_dir = prebuilt_dir.as_ref();

        update_cache(source, prebuilt_dir)?;

        Ok(Self {
            dir: prebuilt_dir.to_owned(),
        })
    }

    /// Get the path of a specific file within the cache.
    pub fn get_file(&self, arch: Arch, file_type: FileType) -> PathBuf {
        self.dir.join(arch.as_str()).join(file_type.as_str())
    }
}

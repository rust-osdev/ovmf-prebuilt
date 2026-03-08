use crate::github::Github;
use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

/// Github added automatic release digests in June 2025:
/// <github.blog/changelog/2025-06-03-releases-now-expose-digests-for-release-assets>
///
/// For older releases, hardcode the SHA-256 digest.
const OLD_RELEASES: [(&str, &str); 6] = [
    (
        "edk2-stable202408-r1",
        "63a9217ddd51fa45d0a89fd83c483cc971765de6bb08e83cf70836b0baff0d48",
    ),
    (
        "edk2-stable202408.01-r1",
        "1b4c7d7603517482a3c4461ba43044c4c7e0a7930274d77eb19600b7dcd9b838",
    ),
    (
        "edk2-stable202411-r1",
        "963fc6cef6a0560cec97381ed22a7d5c76f440c8212529a034cb465466cd57cc",
    ),
    (
        "edk2-stable202502-r1",
        "6d6122e88cdc09e1ffafb6a39fbdbfba668a6ded3f2a032b2cd6c0b7ff6d69df",
    ),
    (
        "edk2-stable202502-r2",
        "dd59d3d52f0a643f07a488f80ab40f89c30f360999d98cdffb30e1eba5476641",
    ),
    (
        "edk2-stable202505-r1",
        "7535021dc7f98453803cefcade59e839e9f559274493d681fec7b19219081e29",
    ),
];

pub fn update_sources() -> Result<()> {
    let sources = get_source_list()?;
    let path = Path::new("ovmf-prebuilt/src/source_constants.rs");
    update_rust_code_in_place(path, &sources)
}

/// Corresponds to the `Source` type in `ovmf-prebuilt/src/lib.rs`.
struct Source {
    tag: String,
    sha256: String,
}

impl Source {
    fn ident(&self) -> String {
        self.tag.to_uppercase().replace(['-', '.'], "_")
    }
}

/// Get the current list of sources.
///
/// This downloads the release list from Github, with really old
/// releases excluded. For each release, the SHA-256 digest is retried
/// from either the github API or from `OLD_RELEASES`. The output is
/// sorted from oldest to newest.
fn get_source_list() -> Result<Vec<Source>> {
    let github = Github::new();

    let releases = github.get_releases()?;

    let mut sources = Vec::new();

    for release in &releases {
        // Exclude really old releases.
        if release.tag_name.as_str() < "edk2-stable202408-r1" {
            break;
        }

        let sha256 = if let Ok(sha256) = release.sha256() {
            sha256
        } else if let Some(old_release) = OLD_RELEASES
            .iter()
            .find(|(name, _digest)| *name == release.tag_name.as_str())
        {
            old_release.1
        } else {
            bail!("missing SHA-256 digest for {}", release.tag_name);
        };

        sources.push(Source {
            tag: release.tag_name.clone(),
            sha256: sha256.to_owned(),
        });
    }

    sources.reverse();

    Ok(sources)
}

fn format_source_list(sources: &[Source]) -> String {
    let mut output = String::new();
    output += "use crate::Source;

#[allow(missing_docs)]
impl Source {
";
    for source in sources {
        output += &format!(
            "    pub const {ident}: Self = Self {{
        tag: \"{tag}\",
        sha256: \"{sha256}\",
    }};

",
            ident = source.ident(),
            tag = source.tag,
            sha256 = source.sha256
        );
    }
    output += &format!(
        "    /// Latest release tag.
    ///
    /// Note that this is not necessarily the latest prebuilt available
    /// from the git repo.
    pub const LATEST: Self = Self::{latest};
}}
",
        latest = sources.last().unwrap().ident()
    );
    output
}

fn update_rust_code_in_place(path: &Path, sources: &[Source]) -> Result<()> {
    let orig_contents = fs::read_to_string(path)?;
    let new_contents = format_source_list(sources);

    if orig_contents == new_contents {
        println!("no changes");
    } else {
        println!("writing updates to {}", path.display());
        fs::write(path, new_contents)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_source_list() {
        let sources = [
            Source {
                tag: "tag-A".to_string(),
                sha256: "1234".to_string(),
            },
            Source {
                tag: "tag-B".to_string(),
                sha256: "5678".to_string(),
            },
        ];

        let expected = "use crate::Source;

#[allow(missing_docs)]
impl Source {
    pub const TAG_A: Self = Self {
        tag: \"tag-A\",
        sha256: \"1234\",
    };

    pub const TAG_B: Self = Self {
        tag: \"tag-B\",
        sha256: \"5678\",
    };

    /// Latest release tag.
    ///
    /// Note that this is not necessarily the latest prebuilt available
    /// from the git repo.
    pub const LATEST: Self = Self::TAG_B;
}
";

        assert_eq!(format_source_list(&sources), expected);
    }
}

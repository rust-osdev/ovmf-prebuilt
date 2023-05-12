use anyhow::{anyhow, bail, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Eq, PartialEq)]
struct Release {
    prebuilt_git_tag: String,
    edk2_git_tag: String,
    release_number: u32,
}

impl Release {
    /// The tag should look something like "edk2-stable202211-r1". The
    /// first part, "edk2-stable202211", should match a tag in the edk2
    /// repo. The "-r1" at the end is so that we can do multiple releases
    /// of the same edk2 tag without overwriting previous ones (e.g. if
    /// we realize later we need to modify a build flag).
    fn from_tag(tag: &str) -> Result<Self> {
        let parts: Vec<_> = tag.rsplitn(2, '-').collect();
        let edk2_git_tag = parts[1];
        if !edk2_git_tag.starts_with("edk2-") {
            bail!("bad edk2 git tag");
        }
        let release_number = parts[0]
            .strip_prefix('r')
            .ok_or(anyhow!("bad release number"))?;
        Ok(Self {
            prebuilt_git_tag: tag.to_string(),
            edk2_git_tag: edk2_git_tag.to_string(),
            release_number: release_number.parse()?,
        })
    }

    /// Get the tarball name based off the git tag.
    fn tarball_name(&self) -> String {
        format!("{}-bin.tar.xz", self.prebuilt_git_tag)
    }

    /// Check if this release has already been pushed.
    fn exists(&self) -> bool {
        let mut cmd = Command::new("gh");
        cmd.arg("release").arg("view").arg(&self.prebuilt_git_tag);
        cmd.status().unwrap().success()
    }

    /// Push the tarball as a new release.
    fn push(&self) -> Result<()> {
        let release_notes = format!(
            "edk2 tag: https://github.com/tianocore/edk2/releases/tag/{}",
            self.edk2_git_tag
        );

        println!("Creating release {}", self.prebuilt_git_tag);
        let mut cmd = Command::new("gh");
        cmd.args(["release", "create", &self.prebuilt_git_tag])
            .args(["--title", &self.prebuilt_git_tag])
            .args(["--notes", &release_notes])
            .arg(self.tarball_name());
        let status = cmd.status()?;
        if !status.success() {
            bail!("gh release failed")
        }
        Ok(())
    }
}

fn build_tarball(opt: &Opt, release: &Release) -> Result<PathBuf> {
    let container_tag = "ovmf_prebuilt";

    // Build everything.
    let mut cmd = Command::new(&opt.container_cmd);
    cmd.args([
        "build",
        "-t",
        container_tag,
        "--build-arg",
        &format!("git_tag={}", release.edk2_git_tag),
        "--build-arg",
        &format!("bin_dir={}-bin", release.prebuilt_git_tag),
        ".",
    ]);
    println!("run: {cmd:?}");
    let status = cmd.status()?;
    if !status.success() {
        bail!("command failed: {status:?}");
    }

    // Copy out the tarball from the image.
    let tarball_name = release.tarball_name();
    let mut cmd = Command::new(&opt.container_cmd);
    cmd.args(["run", container_tag, "cat", &tarball_name]);
    println!("run: {cmd:?}");
    let output = cmd.output()?;
    if !output.status.success() {
        bail!("command failed: {:?}", output.status);
    }
    fs::write(&tarball_name, output.stdout)?;

    Ok(tarball_name.into())
}

#[derive(Parser)]
struct Opt {
    /// Base command used to build a container.
    #[arg(long, default_value = "docker")]
    container_cmd: String,

    /// Push a new release to the repo.
    #[arg(long)]
    create_release: bool,

    /// Git tag to release.
    tag: String,
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    let release = Release::from_tag(&opt.tag)?;

    build_tarball(&opt, &release)?;

    if opt.create_release {
        if release.exists() {
            println!("Release {} already exists", release.prebuilt_git_tag);
        } else {
            release.push()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_from_tag() {
        assert_eq!(
            Release::from_tag("edk2-stable202211-r2").unwrap(),
            Release {
                prebuilt_git_tag: "edk2-stable202211-r2".to_string(),
                edk2_git_tag: "edk2-stable202211".to_string(),
                release_number: 2,
            }
        );
    }
}

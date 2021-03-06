use std::path::Path;

fn main() {
    let url = "https://www.kraxel.org/repos/jenkins/edk2/";
    let body = ureq::get(url).call().unwrap().into_string().unwrap();
    let parsed = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("a").unwrap();

    let file_name = parsed
        .select(&selector)
        .map(|e| e.value().attr("href").unwrap())
        .find(|link| link.contains("git-ovmf-x64"))
        .expect("no ovmf link found");
    println!("Downloading {}", file_name);
    let (date, build_number, hash) = {
        let stripped = file_name.strip_prefix("edk2.git-ovmf-x64-0-").unwrap();
        let mut components = stripped.split('.');
        let date = components.next().unwrap();
        let build_number = components.next().unwrap();
        let hash = components.next().unwrap();
        (date, build_number, hash)
    };

    let target_dir = Path::new("target").join("download");
    std::fs::create_dir_all(&target_dir).unwrap();

    let mut download = ureq::get(&(url.to_string() + file_name))
        .call()
        .unwrap()
        .into_reader();
    let target_file_path = target_dir.join(file_name);
    let mut target_file = std::fs::File::create(&target_file_path).unwrap();
    std::io::copy(&mut download, &mut target_file).unwrap();

    let cpio = target_file_path.with_extension("cpio");
    if cpio.exists() {
        std::fs::remove_file(&cpio).unwrap();
    }
    let extracted = target_dir.join("extracted");
    if extracted.exists() {
        std::fs::remove_dir_all(&extracted).unwrap();
    }

    let mut extract_rpm = std::process::Command::new("7z");
    extract_rpm.arg("x");
    extract_rpm.arg(target_file_path);
    extract_rpm.arg(format!("-o{}", target_dir.display()));
    if !extract_rpm.status().unwrap().success() {
        panic!("rpm extraction failed");
    }

    let mut extract_cpio = std::process::Command::new("7z");
    extract_cpio.arg("x");
    extract_cpio.arg(&cpio);
    extract_cpio.arg(format!("-o{}", extracted.display()));
    if !extract_cpio.status().unwrap().success() {
        panic!("cpio extraction failed");
    }

    let ovmf_root = extracted
        .join("usr")
        .join("share")
        .join("edk2.git")
        .join("ovmf-x64");
    assert!(ovmf_root.exists());

    if std::env::var("CI").as_deref() == Ok("true") {
        let version = format!("v0.{}.{}+{}", date, build_number, hash);

        let exists = {
            let mut cmd = std::process::Command::new("gh");
            cmd.arg("release").arg("view").arg(&version);
            cmd.status().unwrap().success()
        };

        if exists {
            println!("Version {} was already released", version);
        } else {
            println!("Releasing version {}", version);
            let mut cmd = std::process::Command::new("gh");
            cmd.arg("release").arg("create").arg(&version);
            for entry in std::fs::read_dir(&ovmf_root).unwrap() {
                cmd.arg(entry.unwrap().path());
            }
            if !cmd.status().unwrap().success() {
                panic!("gh release failed")
            }
        }
    }
}

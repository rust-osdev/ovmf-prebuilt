use std::path::{PathBuf, Path};

pub fn ovmf_pure_efi() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("OVMF-pure-efi.fd");
    assert!(path.exists(), "OVMF-pure-efi.fd does not exist");
    path
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ovmf_pure_efi() {
        crate::ovmf_pure_efi();
    }
}

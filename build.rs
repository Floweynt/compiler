fn main() {
    for entry in walkdir::WalkDir::new("src") {
        let path = entry.unwrap().path().to_path_buf();
        if path.extension().and_then(|f| f.to_str()) == Some("nodes") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

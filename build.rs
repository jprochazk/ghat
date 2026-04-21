use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    rerun_if_ts_files_changed(Path::new("src/runtime"));
}

fn rerun_if_ts_files_changed(dir: &Path) {
    let entries = fs::read_dir(dir).expect("failed to read src/runtime");

    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();

        if path.is_dir() {
            rerun_if_ts_files_changed(&path);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("ts") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

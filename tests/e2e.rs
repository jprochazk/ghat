use std::fs;
use std::path::Path;
use std::process::Command;

fn run_fixture(fixture_dir: &Path) {
    let fixture_name = fixture_dir.file_name().unwrap().to_str().unwrap();
    let workflows_src = fixture_dir.join("workflows");
    assert!(
        workflows_src.is_dir(),
        "fixture {fixture_name} is missing a workflows/ directory"
    );

    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let workflows_dst = tmp.path().join(".github/ghat/workflows");
    copy_dir_recursive(&workflows_src, &workflows_dst);

    let output = Command::new(env!("CARGO_BIN_EXE_ghat"))
        .arg("generate")
        .current_dir(tmp.path())
        .output()
        .expect("failed to run ghat");

    let mut snapshot = String::new();

    // Exit code
    let code = output.status.code().unwrap_or(-1);
    snapshot.push_str(&format!("exit_code: {code}\n"));

    // Stderr (scrub temp paths)
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.replace(tmp.path().to_str().unwrap(), "[TMP]");
    if !stderr.is_empty() {
        snapshot.push_str(&format!("---stderr---\n{stderr}"));
        if !stderr.ends_with('\n') {
            snapshot.push('\n');
        }
    }

    // Collect generated yaml files
    let output_dir = tmp.path().join(".github/workflows");
    if output_dir.is_dir() {
        let mut files: Vec<_> = fs::read_dir(&output_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
            })
            .collect();
        files.sort_by_key(|e| e.file_name());

        for entry in files {
            let name = entry.file_name().to_string_lossy().to_string();
            let content = fs::read_to_string(entry.path()).unwrap();
            snapshot.push_str(&format!("---{name}---\n{content}"));
            if !content.ends_with('\n') {
                snapshot.push('\n');
            }
        }
    }

    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(Path::new("snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(fixture_name, snapshot);
    });
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let target = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_recursive(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

#[glob_test::glob("./fixtures/*")]
fn fixture(path: &Path) {
    run_fixture(path);
}

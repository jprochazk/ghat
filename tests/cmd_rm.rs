mod support;

use support::TestProject;

fn project_with_lockfile(content: &str) -> TestProject {
    TestProject::new()
        .init()
        .file(".github/ghat/ghat.lock", content)
        .build()
}

/// Build a project with two actions in the lockfile and pre-populated codegen files.
fn project_with_two_actions() -> TestProject {
    TestProject::new()
        .init()
        .file(".github/ghat/ghat.lock", TWO_ACTIONS)
        .file(".github/ghat/actions/actions__checkout.d.ts", "// checkout types\n")
        .file(".github/ghat/actions/Swatinem__rust-cache.d.ts", "// rust-cache types\n")
        .file(".github/ghat/actions/cache.json", &format!(
            r#"{{"Swatinem/rust-cache@9d47c6ad4b02e050fd481d890b2ea34778fd09d6":{{"name":"Rust Cache","inputs":{{"shared-key":{{"description":"An additional key"}},"cache-on-failure":{{"description":"Cache on failure"}}}},"outputs":{{"cache-hit":{{"description":"Whether hit"}}}}}},"actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683":{{"name":"Checkout","inputs":{{"fetch-depth":{{"description":"Depth"}}}},"outputs":{{"ref":{{"description":"Ref"}}}}}}}}"#
        ))
        .file(".github/ghat/actions/mappings.js", "// old mappings\n")
        .build()
}

const TWO_ACTIONS: &str = "\
Swatinem/rust-cache tag:v2.7.8 9d47c6ad4b02e050fd481d890b2ea34778fd09d6
actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683
";

#[test]
fn rm_single() {
    let p = project_with_two_actions();
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = p.ghat(&["rm", "actions/checkout"]).run();
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
    // .d.ts for checkout should be deleted
    assert!(!p.file_exists(".github/ghat/actions/actions__checkout.d.ts"));
    // rust-cache .d.ts should remain
    assert!(p.file_exists(".github/ghat/actions/Swatinem__rust-cache.d.ts"));
    // mappings.js should be regenerated
    snapshot!("mappings", p.read_file(".github/ghat/actions/mappings.js"));
}

#[test]
fn rm_multiple() {
    let p = project_with_two_actions();
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = p
        .ghat(&["rm", "actions/checkout", "Swatinem/rust-cache"])
        .run();
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
    assert!(!p.file_exists(".github/ghat/actions/actions__checkout.d.ts"));
    assert!(!p.file_exists(".github/ghat/actions/Swatinem__rust-cache.d.ts"));
}

#[test]
fn rm_not_found() {
    let p = project_with_lockfile(TWO_ACTIONS);
    let output = p.ghat(&["rm", "nonexistent/action"]).run();
    snapshot!(output);
}

#[test]
fn rm_with_suggestion() {
    let p = project_with_lockfile(TWO_ACTIONS);
    let output = p.ghat(&["rm", "actions/checkotu"]).run();
    snapshot!(output);
}

#[test]
fn rm_partial_failure() {
    let p = project_with_lockfile(TWO_ACTIONS);
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = p
        .ghat(&["rm", "actions/checkout", "nonexistent/action"])
        .run();
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    // All-or-nothing: lockfile should be unchanged
    let diff = before.diff(&after);
    assert!(diff.is_empty(), "expected no diff, got:\n{diff}");
}

#[test]
fn rm_no_actions() {
    let p = project_with_lockfile(TWO_ACTIONS);
    let output = p.ghat(&["rm"]).run();
    snapshot!(output);
}

#[test]
fn rm_no_lockfile() {
    let p = TestProject::new().build();
    let output = p.ghat(&["rm", "actions/checkout"]).run();
    snapshot!(output);
}

mod support;

use support::TestProject;

fn project_with_lockfile(content: &str) -> TestProject {
    TestProject::new()
        .init()
        .file(".github/ghat/ghat.lock", content)
        .build()
}

const TWO_ACTIONS: &str = "\
Swatinem/rust-cache v2.7.8 9d47c6ad4b02e050fd481d890b2ea34778fd09d6
actions/checkout v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683
";

#[test]
fn rm_single() {
    let p = project_with_lockfile(TWO_ACTIONS);
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = p.ghat(&["rm", "actions/checkout"]).run();
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
}

#[test]
fn rm_multiple() {
    let p = project_with_lockfile(TWO_ACTIONS);
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = p.ghat(&["rm", "actions/checkout", "Swatinem/rust-cache"]).run();
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
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

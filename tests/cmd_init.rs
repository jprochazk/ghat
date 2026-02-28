mod support;

use support::TestProject;

#[test]
fn fresh_project() {
    let p = TestProject::new().build();
    let output = p.ghat(&["init"]).run();

    snapshot!("output", output);
    snapshot!("project", p.snapshot_full());
}

#[test]
fn idempotent() {
    let p = TestProject::new().build();

    let out1 = p.ghat(&["init"]).run();
    assert_eq!(out1.exit_code, 0);
    let first = p.snapshot_full();

    let out2 = p.ghat(&["init"]).run();
    assert_eq!(out2.exit_code, 0);
    let second = p.snapshot_full();

    // No diff between first and second init
    let diff = first.diff(&second);
    assert!(diff.is_empty(), "expected no diff, got:\n{diff}");
}

#[test]
fn updates_type_defs() {
    let p = TestProject::new().init().build();
    let before = p.snapshot_full();

    // Corrupt a type definition file
    std::fs::write(p.path().join(".github/ghat/types/api.d.ts"), "// modified").unwrap();

    // Re-running init should restore it
    let output = p.ghat(&["init"]).run();
    assert_eq!(output.exit_code, 0);
    let after = p.snapshot_full();

    // Should be identical to original
    let diff = before.diff(&after);
    assert!(diff.is_empty(), "expected no diff, got:\n{diff}");
}

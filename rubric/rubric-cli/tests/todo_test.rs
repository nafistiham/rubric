use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn rubric_bin() -> std::path::PathBuf {
    let bin = env!("CARGO_BIN_EXE_rubric-cli");
    std::path::PathBuf::from(bin)
}

fn write_rb(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

fn write_toml(dir: &TempDir, content: &str) {
    fs::write(dir.path().join("rubric.toml"), content).unwrap();
}

// ── test_todo_generates_file ───────────────────────────────────────────────────

#[test]
fn test_todo_generates_file() {
    let dir = TempDir::new().unwrap();
    // TrailingWhitespace is always enabled by default — easy violation to trigger.
    write_rb(&dir, "a.rb", "x = 1   \n");
    write_rb(&dir, "b.rb", "y = 2   \n");

    let out = Command::new(rubric_bin())
        .args(["todo", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "rubric todo should exit 0; got: {stdout}");

    let todo_path = dir.path().join(".rubric_todo.toml");
    assert!(todo_path.exists(), ".rubric_todo.toml should be created");

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(
        content.contains("Layout/TrailingWhitespace"),
        "expected Layout/TrailingWhitespace in todo file, got:\n{content}"
    );
    assert!(content.contains("a.rb"), "expected a.rb in todo file");
    assert!(content.contains("b.rb"), "expected b.rb in todo file");
}

// ── test_todo_suppresses_known_violations ─────────────────────────────────────

#[test]
fn test_todo_suppresses_known_violations() {
    let dir = TempDir::new().unwrap();
    write_rb(&dir, "a.rb", "x = 1   \n");

    // Generate the todo file first
    Command::new(rubric_bin())
        .args(["todo", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(dir.path().join(".rubric_todo.toml").exists());

    // Now check — all violations should be suppressed
    let out = Command::new(rubric_bin())
        .args(["check", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "rubric check should succeed with todo suppression; stdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stdout.contains("No violations found"),
        "expected 'No violations found', got: {stdout}"
    );
    assert!(
        stderr.contains("suppressed"),
        "expected suppressed count in stderr, got: {stderr}"
    );
}

// ── test_todo_surfaces_new_violations ─────────────────────────────────────────

#[test]
fn test_todo_surfaces_new_violations() {
    let dir = TempDir::new().unwrap();
    // Start with only trailing whitespace
    write_rb(&dir, "a.rb", "x = 1   \n");

    // Generate todo (suppresses TrailingWhitespace)
    Command::new(rubric_bin())
        .args(["todo", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    // Now add a new violation type that is NOT in the todo: add a second file with
    // a different cop. Use EndOfLine = CRLF in a file not covered by the todo.
    // Actually, it's simpler to add a Style/Documentation violation by putting a class
    // without a comment — but that cop might be disabled. Use a file with trailing
    // whitespace that was NOT in the original todo run.
    write_rb(&dir, "new_file.rb", "class Foo   \nend\n");

    let out = Command::new(rubric_bin())
        .args(["check", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // new_file.rb has TrailingWhitespace but it's not in the todo → should surface
    assert!(
        !out.status.success(),
        "rubric check should fail for new violations not in todo; stdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stdout.contains("new_file.rb"),
        "expected new_file.rb in output, got: {stdout}"
    );
}

// ── test_ignore_todo_flag ──────────────────────────────────────────────────────

#[test]
fn test_ignore_todo_flag() {
    let dir = TempDir::new().unwrap();
    write_rb(&dir, "a.rb", "x = 1   \n");

    // Generate todo
    Command::new(rubric_bin())
        .args(["todo", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    // With --ignore-todo, violations should appear even though they're in the todo
    let out = Command::new(rubric_bin())
        .args(["check", "--ignore-todo", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "rubric check --ignore-todo should fail when violations exist; stdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stdout.contains("a.rb"),
        "expected a.rb in output with --ignore-todo, got: {stdout}"
    );
}

// ── test_regenerate_todo_flag ──────────────────────────────────────────────────

#[test]
fn test_regenerate_todo_flag() {
    let dir = TempDir::new().unwrap();
    // Use disabled_by_default so only TrailingWhitespace runs — keeps the test deterministic
    write_toml(
        &dir,
        "[linter]\ndisabled_by_default = true\n\n[rules.\"Layout/TrailingWhitespace\"]\nenabled = true\n",
    );
    write_rb(&dir, "a.rb", "x = 1   \n");
    write_rb(&dir, "b.rb", "y = 2   \n");

    // Generate initial todo (both files suppressed under TrailingWhitespace)
    Command::new(rubric_bin())
        .args(["todo", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let initial = fs::read_to_string(dir.path().join(".rubric_todo.toml")).unwrap();
    assert!(initial.contains("a.rb"), "a.rb should be in initial todo");
    assert!(initial.contains("b.rb"), "b.rb should be in initial todo");

    // Fix b.rb (remove trailing whitespace)
    fs::write(dir.path().join("b.rb"), "y = 2\n").unwrap();

    // Regenerate — b.rb should no longer appear since it has no violations
    let out = Command::new(rubric_bin())
        .args(["check", "--regenerate-todo", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "regenerate-todo should exit 0");

    let content = fs::read_to_string(dir.path().join(".rubric_todo.toml")).unwrap();
    assert!(content.contains("a.rb"), "a.rb should still be in todo");
    assert!(!content.contains("b.rb"), "b.rb should be removed from todo after fix");
}

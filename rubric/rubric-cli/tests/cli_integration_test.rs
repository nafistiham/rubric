use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn rubric_bin() -> std::path::PathBuf {
    // CARGO_BIN_EXE_<name> is set by cargo test to the path of the compiled binary.
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

// ── linter.enabled = false ────────────────────────────────────────────────────

#[test]
fn check_respects_linter_enabled_false() {
    let dir = TempDir::new().unwrap();
    // File has a genuine violation (trailing whitespace).
    write_rb(&dir, "a.rb", "x = 1   \n");
    write_toml(&dir, "[linter]\nenabled = false\n");

    let out = Command::new(rubric_bin())
        .args(["check", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "exit code should be 0 when linter disabled");
    assert!(stdout.contains("Linter disabled"), "expected disabled message, got: {stdout}");
}

// ── formatter.enabled = false ─────────────────────────────────────────────────

#[test]
fn fmt_respects_formatter_enabled_false() {
    let dir = TempDir::new().unwrap();
    let original = "x = 1   \n";
    let file = write_rb(&dir, "a.rb", original);
    write_toml(&dir, "[formatter]\nenabled = false\n");

    let out = Command::new(rubric_bin())
        .args(["fmt", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Formatter disabled"), "expected disabled message, got: {stdout}");
    // File must be untouched.
    assert_eq!(fs::read_to_string(&file).unwrap(), original);
}

// ── global excludes in fmt ────────────────────────────────────────────────────

#[test]
fn fmt_respects_global_excludes() {
    let dir = TempDir::new().unwrap();
    let original = "x = 1   \n";
    let file = write_rb(&dir, "vendor.rb", original);
    // Exclude the file globally.
    write_toml(&dir, "exclude = [\"vendor.rb\"]\n");

    Command::new(rubric_bin())
        .args(["fmt", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        original,
        "excluded file should not be modified by fmt"
    );
}

// ── per-rule excludes in fmt ──────────────────────────────────────────────────

#[test]
fn fmt_respects_per_rule_excludes() {
    let dir = TempDir::new().unwrap();
    let original = "x = 1   \n";
    let file = write_rb(&dir, "generated.rb", original);
    // Use disabled_by_default so only TrailingWhitespace runs, then exclude generated.rb
    // from it. That way no other fixable cop can touch the file.
    write_toml(
        &dir,
        "[linter]\ndisabled_by_default = true\n\n\
         [rules.\"Layout/TrailingWhitespace\"]\nenabled = true\nexclude = [\"generated.rb\"]\n",
    );

    Command::new(rubric_bin())
        .args(["fmt", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        original,
        "per-rule excluded file should not be modified by fmt"
    );
}

// ── upward config discovery ───────────────────────────────────────────────────

#[test]
fn check_finds_config_in_ancestor_directory() {
    let root = TempDir::new().unwrap();
    // Config at root disables the linter.
    write_toml(&root, "[linter]\nenabled = false\n");

    // Ruby file lives two levels deep.
    let nested = root.path().join("app").join("models");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("user.rb"), "x = 1   \n").unwrap();

    let out = Command::new(rubric_bin())
        .args(["check", nested.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "should exit 0 when ancestor config disables linter");
    assert!(stdout.contains("Linter disabled"), "should find config in ancestor dir, got: {stdout}");
}

#[test]
fn fmt_finds_config_in_ancestor_directory() {
    let root = TempDir::new().unwrap();
    write_toml(&root, "[formatter]\nenabled = false\n");

    let nested = root.path().join("app").join("views");
    fs::create_dir_all(&nested).unwrap();
    let original = "x = 1   \n";
    let file = nested.join("index.rb");
    fs::write(&file, original).unwrap();

    let out = Command::new(rubric_bin())
        .args(["fmt", nested.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Formatter disabled"), "should find config in ancestor dir, got: {stdout}");
    assert_eq!(fs::read_to_string(&file).unwrap(), original);
}

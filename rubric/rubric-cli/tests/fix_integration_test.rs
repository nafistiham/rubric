use std::fs;
use tempfile::TempDir;

#[test]
fn fix_removes_trailing_whitespace() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.rb");
    fs::write(&file, "def hello   \n  puts 'world'  \nend\n").unwrap();

    use rubric_core::{LintContext, apply_fixes};
    use rubric_rules::TrailingWhitespace;
    use rubric_core::Rule;

    let source = fs::read_to_string(&file).unwrap();
    let ctx = LintContext::new(&file, &source);
    let diags = TrailingWhitespace.check_source(&ctx);
    let fixes: Vec<_> = diags.iter()
        .filter_map(|d| TrailingWhitespace.fix(d))
        .collect();
    let corrected = apply_fixes(&source, &fixes);
    fs::write(&file, &corrected).unwrap();

    let result = fs::read_to_string(&file).unwrap();
    assert_eq!(result, "def hello\n  puts 'world'\nend\n");
}

#[test]
fn fmt_applies_all_safe_fixes_to_file() {
    use rubric_core::{LintContext, apply_fixes, Rule};
    use rubric_rules::TrailingWhitespace;

    let dir = TempDir::new().unwrap();
    let file = dir.path().join("fmt_test.rb");
    fs::write(&file, "x = 1   \ny = 2  \n").unwrap();

    let source = fs::read_to_string(&file).unwrap();
    let ctx = LintContext::new(&file, &source);

    let diags = TrailingWhitespace.check_source(&ctx);
    let fixes: Vec<_> = diags.iter()
        .filter_map(|d| TrailingWhitespace.fix(d))
        .collect();

    assert_eq!(fixes.len(), 2, "expected 2 fixes for 2 lines with trailing whitespace");

    let corrected = apply_fixes(&source, &fixes);
    fs::write(&file, &corrected).unwrap();

    let result = fs::read_to_string(&file).unwrap();
    assert_eq!(result, "x = 1\ny = 2\n");
}

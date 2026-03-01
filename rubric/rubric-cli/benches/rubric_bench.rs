use criterion::{criterion_group, criterion_main, Criterion};
use rubric_rules::*;
use rubric_core::Rule;
use std::path::PathBuf;
use walkdir::WalkDir;

fn build_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(TrailingWhitespace),
        Box::new(LineLength),
        Box::new(IndentationWidth),
        Box::new(TrailingNewlines),
        Box::new(EmptyLines),
        Box::new(SpaceAfterComma),
        Box::new(SpaceBeforeComment),
        Box::new(FrozenStringLiteralComment),
    ]
}

fn collect_ruby_files(root: &str) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rb"))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn bench_lint_rubric_source(c: &mut Criterion) {
    // Find Ruby files in the rubric-rules test fixtures
    let fixture_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../rubric-rules/tests/fixtures");
    let files = collect_ruby_files(fixture_dir);

    if files.is_empty() {
        eprintln!("No Ruby files found for benchmark in {fixture_dir}");
        return;
    }

    let rules = build_rules();

    c.bench_function("lint_fixture_files", |b| {
        b.iter(|| {
            let results: Vec<_> = files
                .iter()
                .filter_map(|path| {
                    let source = std::fs::read_to_string(path).ok()?;
                    let ctx = rubric_core::LintContext::new(path, &source);
                    let mut diags = Vec::new();
                    for rule in &rules {
                        diags.extend(rule.check_source(&ctx));
                    }
                    Some((path.clone(), diags))
                })
                .collect();
            criterion::black_box(results)
        })
    });
}

fn bench_lint_generated_file(c: &mut Criterion) {
    // Generate a synthetic 1000-line Ruby file
    let mut source = String::with_capacity(50_000);
    for i in 0..200 {
        source.push_str(&format!(
            "class Foo{i}\n  def initialize\n    @value = {i}\n  end\n\n  def value\n    @value\n  end\nend\n\n"
        ));
    }

    let path = PathBuf::from("generated.rb");
    let rules = build_rules();

    c.bench_function("lint_generated_2000_lines", |b| {
        b.iter(|| {
            let ctx = rubric_core::LintContext::new(&path, &source);
            let mut diags = Vec::new();
            for rule in &rules {
                diags.extend(rule.check_source(&ctx));
            }
            criterion::black_box(diags)
        })
    });
}

criterion_group!(benches, bench_lint_rubric_source, bench_lint_generated_file);
criterion_main!(benches);

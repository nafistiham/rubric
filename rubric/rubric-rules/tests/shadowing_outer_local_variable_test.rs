use rubric_core::{LintContext, Rule};
use rubric_rules::lint::shadowing_outer_local_variable::ShadowingOuterLocalVariable;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/shadowing_outer_local_variable/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ShadowingOuterLocalVariable.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/ShadowingOuterLocalVariable"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = 1\n[1, 2].each { |y| puts y }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ShadowingOuterLocalVariable.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// ── False positive: `key` in method `foo` must not flag `|key|` in method `bar` ──
#[test]
fn no_false_positive_across_method_boundaries() {
    let src = "\
def foo
  key = 'signal'
  redis.call('SUBSCRIBE', key)
end

def bar
  redis.subscribe do |key, val|
    puts val
  end
end
";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ShadowingOuterLocalVariable.check_source(&ctx);
    assert!(diags.is_empty(), "cross-method shadowing false positive: {:?}", diags);
}

// ── False positive: brace-block params must not leak into outer-local set ───────
#[test]
fn no_false_positive_for_brace_block_variable() {
    let src = "\
def connect
  pool.with { |conn| conn.ping }
  conn = nil
end

def query
  pool.with do |conn|
    conn.exec
  end
end
";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ShadowingOuterLocalVariable.check_source(&ctx);
    assert!(diags.is_empty(), "brace-block param leaked as outer local: {:?}", diags);
}

// ── True positive: genuine shadowing inside the same method still detected ───────
#[test]
fn detects_genuine_shadowing_in_same_method() {
    let src = "\
def process
  name = 'alice'
  list.each do |name|
    puts name
  end
end
";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ShadowingOuterLocalVariable.check_source(&ctx);
    assert!(!diags.is_empty(), "genuine same-method shadowing not detected");
}

// ── False positive: real-world sidekiq api.rb pattern ────────────────────────────
#[test]
fn no_false_positive_across_methods_do_end_block() {
    let src = "\
def signal(queue)
  key = \"queue:#{queue}\"
  redis.call('PUBLISH', key, 'stop')
end

def consume(queue, &block)
  redis.subscribe do |key, val|
    block.call(key, val)
  end
end
";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ShadowingOuterLocalVariable.check_source(&ctx);
    assert!(diags.is_empty(), "api.rb cross-method pattern false positive: {:?}", diags);
}

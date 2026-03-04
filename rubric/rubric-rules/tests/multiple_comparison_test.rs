use rubric_core::{LintContext, Rule};
use rubric_rules::lint::multiple_comparison::MultipleComparison;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/multiple_comparison/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/MultipleComparison"));
}

#[test]
fn no_violation_on_clean() {
    let src = "if x > 1 && x < 10\n  puts 'in range'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_false_positive_for_shovel_operator() {
    let src = "result = []\nresult << \"<tag>\"\nresult << \"</tag>\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "shovel operator << should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_hash_with_angle_brackets() {
    let src = "h = {\"key\" => \"<value>\"}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "hash rocket => with angle brackets in string should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_string_with_angle_brackets() {
    let src = "msg = \"error: <none>\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "angle brackets inside strings should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_hash_rocket_with_string_value() {
    let src = "opts = {:format => \"<html>\"}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "hash rocket with HTML string value should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn still_detects_chained_comparison() {
    let src = "if 1 < x < 10\n  puts 'bad'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "chained comparison 1 < x < 10 must still be flagged"
    );
}

#[test]
fn no_false_positive_for_percent_paren_literal_with_html() {
    // %(...) percent literal containing HTML tags with < and >
    let src = "stamp = Time.now.iso8601\n%(<time class=\"x\" title=\"#{stamp}\">#{stamp}</time>)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "%() literal with HTML tags should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_percent_w_literal_with_angle_brackets() {
    // %w[...] array containing a word with < and >
    let src = "tags = %w[foobar <eviltag/> other]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "%w[] literal with angle brackets should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_in_heredoc_body_with_html() {
    // Lines inside a <<~HTML heredoc that contain HTML tags must not be flagged.
    let src = concat!(
        "def link_html\n",
        "  <<~HTML.squish\n",
        "    <a href=\"#{url}\" class=\"mention\">@<span>user</span></a>\n",
        "  HTML\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "HTML inside a heredoc body should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_in_heredoc_body_with_xml() {
    // Lines inside a <<~XML heredoc that contain XML tags must not be flagged.
    let src = concat!(
        "let(:host_meta) do\n",
        "  <<~XML\n",
        "    <XRD xmlns=\"http://docs.oasis-open.org/ns/xri/xrd-1.0\">\n",
        "      <Link rel=\"lrdd\" template=\"https://example.com/{uri}\"/>\n",
        "    </XRD>\n",
        "  XML\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "XML inside a heredoc body should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_regex_with_named_capture_groups() {
    // Regex named capture groups use <name> syntax which contains < and >.
    let src = "PARAM_RE = /(?<key>[a-z]+)=(?<value>[a-z]+)/\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "regex with named capture groups should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_heredoc_with_squish_suffix() {
    // <<~HTML.squish is a common Rails pattern; body lines with HTML must be skipped.
    let src = concat!(
        "def render_tag\n",
        "  <<~HTML.squish\n",
        "    <span class=\"h-card\"><a href=\"#{url}\">@<span>#{name}</span></a></span>\n",
        "  HTML\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "HTML inside <<~HTML.squish heredoc body should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn still_detects_chained_comparison_outside_heredoc() {
    // A real chained comparison that is NOT inside a heredoc must still be flagged.
    let src = concat!(
        "def check(x)\n",
        "  if 0 < x < 100\n",
        "    true\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "chained comparison outside heredoc must still be flagged"
    );
}

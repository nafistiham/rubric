use rubric_core::{LintContext, Rule};
use rubric_rules::lint::useless_assignment::UselessAssignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/useless_assignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/useless_assignment/corrected.rb");

#[test]
fn detects_useless_assignment() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for unused variable, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UselessAssignment"));
}

#[test]
fn no_violation_with_all_vars_used() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── False positive: variable assigned with `{...}` block result then used ─────
#[test]
fn no_false_positive_for_curly_block_assignment_then_used() {
    let src = concat!(
        "def queues\n",
        "  Sidekiq.redis do |conn|\n",
        "    queues = conn.sscan('queues').to_a\n",
        "\n",
        "    lengths = conn.pipelined { |pipeline|\n",
        "      queues.each do |queue|\n",
        "        pipeline.llen(queue)\n",
        "      end\n",
        "    }\n",
        "\n",
        "    array_of_arrays = queues.zip(lengths).sort_by { |_, size| -size }\n",
        "    array_of_arrays.to_h\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "curly block assignment falsely flagged: {:?}", diags);
}

// ── False positive: variable assigned via inline `case` then used ─────────────
#[test]
fn no_false_positive_for_inline_case_assignment() {
    let src = "def force_shutdown_after(val)\n  i = case val\n      when :forever\n        -1\n      else\n        Float(val)\n      end\n  @options[:shutdown] = i\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "inline case assignment falsely flagged: {:?}", diags);
}

// ── False positive: variable used in string interpolation ────────────────────
#[test]
fn no_false_positive_for_string_interpolation_usage() {
    let src = "def build_url(opts)\n  tls_str = opts[:tls] ? '&tls=true' : ''\n  \"ssl://host?#{tls_str}&other\"\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "string interpolation usage falsely flagged: {:?}", diags);
}

// ── False positive: `=~` regex match operator treated as assignment ───────────
#[test]
fn no_false_positive_for_regex_match_operator() {
    let src = concat!(
        "def record_integer_id_request?\n",
        "  second_segment =~ /\\d/\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`=~` falsely flagged as useless assignment: {:?}",
        diags
    );
}

// ── False positive: var = begin...end block then used ────────────────────────
#[test]
fn no_false_positive_for_inline_begin_assignment() {
    let src = concat!(
        "def masked_ip(request)\n",
        "  masked_ip_addr = begin\n",
        "    ip_addr = IPAddr.new(request.remote_ip)\n",
        "    if ip_addr.ipv6?\n",
        "      ip_addr.mask(IPV6_TOLERANCE_MASK)\n",
        "    else\n",
        "      ip_addr.mask(IPV4_TOLERANCE_MASK)\n",
        "    end\n",
        "  end\n",
        "  \"#{masked_ip_addr}/#{masked_ip_addr.prefix}\"\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "inline begin-end assignment falsely flagged: {:?}",
        diags
    );
}

// ── False positive: var = if...end block then used ───────────────────────────
#[test]
fn no_false_positive_for_inline_if_assignment() {
    let src = concat!(
        "def domain_not_allowed?(uri_or_domain)\n",
        "  return false if uri_or_domain.blank?\n",
        "\n",
        "  domain = if uri_or_domain.include?('://')\n",
        "             Addressable::URI.parse(uri_or_domain).host\n",
        "           else\n",
        "             uri_or_domain\n",
        "           end\n",
        "\n",
        "  if limited_federation_mode?\n",
        "    !DomainAllow.allowed?(domain)\n",
        "  else\n",
        "    DomainBlock.blocked?(domain)\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "inline if-end assignment falsely flagged: {:?}",
        diags
    );
}

// ── False positive: var = case...end then used ───────────────────────────────
#[test]
fn no_false_positive_for_inline_case_rhs_assignment() {
    let src = concat!(
        "def from_elasticsearch\n",
        "  query_builder = begin\n",
        "    if options[:use_searchable_text]\n",
        "      FullQueryBuilder.new(terms_for_query)\n",
        "    else\n",
        "      AutocompleteQueryBuilder.new(terms_for_query)\n",
        "    end\n",
        "  end\n",
        "\n",
        "  records = query_builder.build.limit(10).objects\n",
        "  records\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "inline begin/if assignment then used falsely flagged: {:?}",
        diags
    );
}

// ── False positive: var used inside #{} interpolation outside string ──────────
// (e.g. inside a regex or heredoc body)
#[test]
fn no_false_positive_for_interpolation_outside_string() {
    let src = concat!(
        "def build_regex(letters)\n",
        "  letters_cif = %w[A B C D E F G H]\n",
        "  regex_cif = /^(#{letters_cif.join('|')})-?\\d{7}$/\n",
        "  regex_cif\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "var used in interpolation outside double-string falsely flagged: {:?}",
        diags
    );
}

// ── False positive: var used in #{} inside heredoc body ──────────────────────
#[test]
fn no_false_positive_for_var_in_heredoc_interpolation() {
    let src = concat!(
        "def render_html(account)\n",
        "  display_username = account.pretty_acct\n",
        "  <<~HTML\n",
        "    <span>#{display_username}</span>\n",
        "  HTML\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "var used in heredoc interpolation falsely flagged: {:?}",
        diags
    );
}

// ── False positive: var assigned inside begin/rescue, used after ──────────────
#[test]
fn no_false_positive_for_begin_rescue_assignment_then_used() {
    let src = concat!(
        "def iban(country_code: 'GB')\n",
        "  begin\n",
        "    pattern = fetch('bank.iban_details')\n",
        "  rescue I18n::MissingTranslationData\n",
        "    raise ArgumentError, 'not found'\n",
        "  end\n",
        "\n",
        "  account = Base.regexify(/#{pattern}/)\n",
        "  account\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "begin/rescue assignment then used falsely flagged: {:?}",
        diags
    );
}

// ── False positive: var assigned in do-block, used after ─────────────────────
#[test]
fn no_false_positive_for_assignment_in_stub_block_then_used() {
    let src = concat!(
        "def test_hex_color\n",
        "  @tester.stub :hsl_color, mock do\n",
        "    result = @tester.hex_color(hue: 100)\n",
        "    assert_match(/^#[0-9a-f]{6}$/, result)\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "assignment in stub block then used falsely flagged: {:?}",
        diags
    );
}

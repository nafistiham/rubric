use rubric_core::{LintContext, Rule};
use rubric_rules::layout::rescue_ensure_alignment::RescueEnsureAlignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/rescue_ensure_alignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/rescue_ensure_alignment/corrected.rb");

#[test]
fn detects_misaligned_rescue() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/RescueEnsureAlignment"));
}

#[test]
fn no_violation_for_aligned_rescue() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// `rescue` inside a `do...end` block must align with the `do` line's indent,
// not with the enclosing `def`
#[test]
fn no_false_positive_for_rescue_inside_do_block() {
    let src = concat!(
        "  def fields\n",
        "    (self[:fields] || []).filter_map do |f|\n",
        "      Account::Field.new(self, f)\n",
        "    rescue\n",
        "      nil\n",
        "    end\n",
        "  end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "rescue inside do block falsely flagged: {:?}", diags);
}

// `rescue` inside a do block that contains an if/unless block — the if/unless
// `end` must not corrupt the stack so that rescue is checked against the wrong opener.
#[test]
fn no_false_positive_for_rescue_in_do_block_with_inner_if() {
    // Pattern from mastodon move_worker.rb:
    //   def carry_blocks_over!
    //     some_collection.find_each do |item|
    //       if condition
    //         do_something
    //       end
    //     rescue => e
    //       handle(e)
    //     end
    //   end
    let src = concat!(
        "  def carry_blocks_over!\n",
        "    @source_account.find_each do |block|\n",
        "      unless skip?(block)\n",
        "        do_action(block)\n",
        "      end\n",
        "    rescue => e\n",
        "      @deferred_error = e\n",
        "    end\n",
        "  end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "rescue in do block with inner if falsely flagged: {:?}", diags);
}

// `rescue` that is method-level (inside def...end, no explicit begin) with
// an inner begin block that has its own rescue — the inner rescue must not
// pop the method-level do-block frame.
#[test]
fn no_false_positive_for_rescue_in_find_each_with_nested_begin() {
    // Pattern from mastodon move_worker.rb copy_account_notes!
    let src = concat!(
        "  def copy_account_notes!\n",
        "    @source.find_each do |note|\n",
        "      if condition\n",
        "        begin\n",
        "          do_action(note)\n",
        "        rescue ActiveRecord::RecordInvalid\n",
        "          fallback(note)\n",
        "        end\n",
        "      else\n",
        "        update(note)\n",
        "      end\n",
        "    rescue ActiveRecord::RecordInvalid\n",
        "      nil\n",
        "    rescue => e\n",
        "      @deferred_error = e\n",
        "    end\n",
        "  end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "rescue in find_each with nested begin falsely flagged: {:?}", diags);
}

// `var = begin ... rescue ... end` — inline begin assignment.
// The `rescue` should align with the line containing `= begin`, not with the
// enclosing `def` or `do` block.
#[test]
fn no_false_positive_for_inline_begin_assignment() {
    // Pattern from mastodon dashboard_helper.rb, tag.rb, etc.
    let src = concat!(
        "def relevant_account_ip(account, ip)\n",
        "  matched_ip = begin\n",
        "    IPAddr.new(ip)\n",
        "  rescue IPAddr::Error\n",
        "    nil\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "inline begin assignment rescue falsely flagged: {:?}", diags);
}

// `var ||= begin ... rescue ... end` — compound assignment with inline begin.
#[test]
fn no_false_positive_for_compound_assign_begin() {
    // Pattern from mastodon elasticsearch_check.rb: @var ||= begin ... rescue ... end
    let src = concat!(
        "def running_version\n",
        "  @running_version ||= begin\n",
        "    Chewy.client.info['version']['number']\n",
        "  rescue Faraday::ConnectionFailed\n",
        "    nil\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "compound assignment begin rescue falsely flagged: {:?}", diags);
}

// `var = begin` inside a `do...end` block (e.g., inside find_each).
#[test]
fn no_false_positive_for_inline_begin_inside_do_block() {
    // Pattern from mastodon 20240307180905_migrate_devise_two_factor_secrets.rb
    let src = concat!(
        "def up\n",
        "  users.find_each do |user|\n",
        "    otp_secret = begin\n",
        "      user.otp_secret\n",
        "    rescue OpenSSL::OpenSSLError\n",
        "      next\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "inline begin inside do block rescue falsely flagged: {:?}", diags);
}

// `var = if cond` inline if inside a do block — the `end` for the `if` must
// not pop the do block frame, causing rescue to be compared against the wrong opener.
#[test]
fn no_false_positive_for_inline_if_inside_do_block_with_rescue() {
    // Pattern from mastodon email_domain_block.rb: extract_uris
    //   def extract_uris(domain_or_domains)
    //     Array(domain_or_domains).map do |str|
    //       domain = if str.include?('@')
    //                  str.split('@', 2).last
    //                else
    //                  str
    //                end
    //       do_work(domain)
    //     rescue SomeError
    //       nil
    //     end
    //   end
    let src = concat!(
        "  def extract_uris(domain_or_domains)\n",
        "    Array(domain_or_domains).map do |str|\n",
        "      domain = if str.include?('@')\n",
        "                 str.split('@', 2).last\n",
        "               else\n",
        "                 str\n",
        "               end\n",
        "\n",
        "      URI.new(domain)\n",
        "    rescue URI::InvalidURIError\n",
        "      nil\n",
        "    end\n",
        "  end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "inline if inside do block with rescue falsely flagged: {:?}", diags);
}

// `ensure_something` method calls must NOT be treated as `ensure` keywords.
// Only a bare `ensure` keyword (followed by space, end-of-line, or comment) is valid.
#[test]
fn no_false_positive_for_ensure_method_call() {
    // Pattern from mastodon tag_search_service.rb: ensure_exact_match(...)
    let src = concat!(
        "def from_elasticsearch\n",
        "  definition = TagsIndex.query(query)\n",
        "  ensure_exact_match(definition.limit(@limit).objects)\n",
        "rescue Faraday::ConnectionFailed\n",
        "  nil\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "ensure_method_call falsely flagged: {:?}", diags);
}

// `rescue` in a method-level rescue (def...rescue...end pattern) inside a
// define_method block — rescue aligns with define_method's body indent.
#[test]
fn no_false_positive_for_rescue_in_define_method_do_block() {
    // Pattern from mastodon omniauth_callbacks_controller.rb:
    //   def self.provides_callback_for(provider)
    //     define_method provider do
    //       body
    //     rescue ActiveRecord::RecordInvalid
    //       handler
    //     end
    //   end
    let src = concat!(
        "  def self.provides_callback_for(provider)\n",
        "    define_method provider do\n",
        "      @provider = provider\n",
        "      if @user.persisted?\n",
        "        sign_in_and_redirect @user\n",
        "      else\n",
        "        redirect_to new_user_registration_url\n",
        "      end\n",
        "    rescue ActiveRecord::RecordInvalid\n",
        "      flash[:alert] = 'error'\n",
        "    end\n",
        "  end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "rescue in define_method do block falsely flagged: {:?}", diags);
}

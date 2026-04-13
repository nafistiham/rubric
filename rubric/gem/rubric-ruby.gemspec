# frozen_string_literal: true

require_relative "lib/rubric/version"

Gem::Specification.new do |spec|
  spec.name    = "rubric-ruby"
  spec.version = Rubric::VERSION
  spec.authors = ["Rubric Contributors"]
  spec.email   = []

  spec.summary     = "A fast Ruby linter and formatter written in Rust"
  spec.description = "Rubric is a RuboCop-compatible Ruby linter written in Rust " \
                     "— 8-13x faster. Install and run with no Rust toolchain required. " \
                     "The native binary is downloaded automatically on first run."
  spec.homepage    = "https://github.com/nafistiham/rubric"
  spec.license     = "MIT"

  spec.required_ruby_version = ">= 3.0"

  spec.files = Dir[
    "lib/**/*",
    "exe/*",
    "LICENSE",
  ]

  spec.bindir        = "exe"
  spec.executables   = ["rubric"]
  spec.require_paths = ["lib"]

  # No runtime dependencies — the binary is lazily downloaded on first use.
end

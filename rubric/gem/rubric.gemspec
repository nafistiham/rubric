# frozen_string_literal: true

require_relative "lib/rubric/version"

Gem::Specification.new do |spec|
  spec.name    = "rubric"
  spec.version = Rubric::VERSION
  spec.authors = ["Rubric Contributors"]
  spec.email   = []

  spec.summary     = "A fast Ruby linter and formatter written in Rust"
  spec.description = "Rubric is a Rubocop-compatible Ruby linter and formatter " \
                     "implemented in Rust for dramatically faster CI times."
  spec.homepage    = "https://github.com/nafistiham/rubric"
  spec.license     = "MIT"

  spec.required_ruby_version = ">= 3.0"

  spec.files = Dir[
    "lib/**/*",
    "exe/*",
    "*.md",
    "LICENSE*"
  ]

  spec.bindir        = "exe"
  spec.executables   = ["rubric"]
  spec.require_paths = ["lib"]

  # Platform-specific gems provide the actual binary
  # This meta-gem selects the right platform gem
  unless RUBY_PLATFORM =~ /java/
    spec.add_dependency "rubric-#{Gem::Platform.local}", spec.version
  end
end

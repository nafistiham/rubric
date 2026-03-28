# frozen_string_literal: true

require_relative "rubric/version"

module Rubric
  # Returns the path to the rubric native binary installed by the platform gem,
  # or nil if it cannot be found.
  def self.executable_path
    # The platform gem installs the binary next to this file under
    # rubric-<platform>/exe/rubric, or directly onto PATH via RubyGems binstubs.
    # Walk gem dirs to find it.
    Gem.paths.home # ensure gem home is initialised
    Gem::Specification.each do |spec|
      next unless spec.name.start_with?("rubric-") && spec.name != "rubric-linter"
      exe = File.join(spec.gem_dir, "exe", "rubric")
      return exe if File.executable?(exe)
    end

    # Fallback: check PATH for a `rubric` binary that is NOT this script.
    path_binary = Gem.find_files("../exe/rubric").first
    return path_binary if path_binary && File.executable?(path_binary)

    nil
  end

  # Exec the native binary, replacing the current process.
  # Raises RuntimeError if the binary cannot be located.
  def self.exec!(*args)
    bin = executable_path
    unless bin
      abort "rubric: could not find a native rubric binary.\n" \
            "Install the platform-specific gem, e.g.:\n" \
            "  gem install rubric-linter\n" \
            "or add it to your Gemfile."
    end
    Kernel.exec(bin, *args)
  end
end

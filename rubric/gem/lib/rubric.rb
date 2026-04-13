# frozen_string_literal: true

require_relative "rubric/version"
require "fileutils"
require "open-uri"
require "tmpdir"

module Rubric
  GITHUB_REPO = "nafistiham/rubric"

  # Maps RUBY_PLATFORM patterns → Rust target triple used in release asset names.
  PLATFORM_MAP = {
    /aarch64-apple-darwin/  => "aarch64-apple-darwin",
    /x86_64-apple-darwin/   => "x86_64-apple-darwin",
    /arm64-darwin/          => "aarch64-apple-darwin",
    /x86_64-darwin/         => "x86_64-apple-darwin",
    /aarch64-linux/         => "aarch64-unknown-linux-gnu",
    /x86_64-linux/          => "x86_64-unknown-linux-gnu",
  }.freeze

  # Directory where the downloaded binary is cached (~/.rubric/bin/).
  def self.cache_dir
    dir = File.join(Gem.user_home, ".rubric", "bin")
    FileUtils.mkdir_p(dir)
    dir
  end

  # Path to the cached rubric binary for this platform+version.
  def self.cached_binary
    File.join(cache_dir, "rubric-#{VERSION}-#{rust_target}")
  end

  # Detect the Rust target triple for the current platform.
  def self.rust_target
    platform = RUBY_PLATFORM
    PLATFORM_MAP.each do |pattern, target|
      return target if platform.match?(pattern)
    end

    # Fallback via uname for unusual RUBY_PLATFORM strings.
    arch = `uname -m`.strip
    os   = `uname -s`.strip.downcase
    case [os, arch]
    when ["darwin", "arm64"]  then "aarch64-apple-darwin"
    when ["darwin", "x86_64"] then "x86_64-apple-darwin"
    when ["linux",  "x86_64"] then "x86_64-unknown-linux-gnu"
    when ["linux",  "aarch64"], ["linux", "arm64"] then "aarch64-unknown-linux-gnu"
    else
      abort "rubric: unsupported platform #{platform} (#{os}/#{arch}). " \
            "Please open an issue at https://github.com/#{GITHUB_REPO}/issues"
    end
  end

  # Download the binary for the current platform from GitHub Releases.
  def self.download_binary!
    target  = rust_target
    archive = "rubric-#{VERSION}-#{target}.tar.gz"
    url     = "https://github.com/#{GITHUB_REPO}/releases/download/v#{VERSION}/#{archive}"
    dest    = cached_binary

    warn "rubric: downloading #{archive} from GitHub Releases..."

    Dir.mktmpdir do |tmp|
      archive_path = File.join(tmp, archive)

      begin
        URI.open(url, "rb") do |remote|  # rubocop:disable Security/Open
          File.open(archive_path, "wb") { |f| f.write(remote.read) }
        end
      rescue OpenURI::HTTPError => e
        abort "rubric: download failed (#{e.message})\n" \
              "URL: #{url}\n" \
              "Check https://github.com/#{GITHUB_REPO}/releases for available versions."
      end

      unless system("tar", "-xzf", archive_path, "-C", tmp)
        abort "rubric: failed to extract #{archive}"
      end

      rubric_bin = File.join(tmp, "rubric")
      abort "rubric: binary not found in archive" unless File.exist?(rubric_bin)

      FileUtils.cp(rubric_bin, dest)
      FileUtils.chmod(0o755, dest)
    end

    warn "rubric: cached to #{dest}"
    dest
  end

  # Return the path to a working rubric binary, downloading if necessary.
  def self.executable_path
    bin = cached_binary
    return bin if File.executable?(bin)

    download_binary!
  end

  # Exec the native binary, replacing the current process.
  def self.exec!(*args)
    Kernel.exec(executable_path, *args)
  end
end

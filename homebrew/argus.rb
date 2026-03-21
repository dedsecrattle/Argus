class Argus < Formula
  desc "Production-ready web crawler capable of handling billions of URLs"
  homepage "https://github.com/dedsecrattle/argus"
  url "https://github.com/dedsecrattle/argus/archive/refs/tags/v{{ version }}.tar.gz"
  sha256 "{{ tarball_sha256 }}"
  license "MIT"

  depends_on "rust" => :build
  depends_on "redis" => :optional

  def install
    system "cargo", "install", "--path", ".", "--root", prefix
  end

  test do
    system "#{bin}/argus", "--version"
    system "#{bin}/argus", "help"
  end
end

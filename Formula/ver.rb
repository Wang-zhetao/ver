class Ver < Formula
  desc "High-performance Node.js version manager written in Rust"
  homepage "https://github.com/yourusername/ver"
  url "https://github.com/yourusername/ver/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/ver", "--version"
  end
end 

class Adot < Formula
  desc "A minimal dotfile manager"
  homepage "https://github.com/Dimfred/adot"
  url "https://github.com/Dimfred/adot/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "adot", shell_output("#{bin}/adot --help")
  end
end

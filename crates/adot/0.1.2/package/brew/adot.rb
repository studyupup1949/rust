class Adot < Formula
  desc "A minimal dotfile manager"
  homepage "https://github.com/Dimfred/adot"
  url "https://github.com/Dimfred/adot/archive/refs/tags/v0.1.1.tar.gz"
  sha256 "d5558cd419c8d46bdc958064cb97f963d1ea793866414c025906ec15033512ed"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "adot", shell_output("#{bin}/adot --help")
  end
end

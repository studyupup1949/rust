class Adot < Formula
  desc "A minimal dotfile manager"
  homepage "https://github.com/Dimfred/adot"
  version "0.1.10"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Dimfred/adot/releases/download/v0.1.10/macos-adot-v0.1.10-arm64"
      sha256 "8c819ba524a9c0b3cc6186caf95ebbca3a741156bf789b1577073426584e412e"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Dimfred/adot/releases/download/v0.1.10/linux-adot-v0.1.10-x86_64"
      sha256 "a50e9b62bbc5209e238b108c95c8acd561e8d78e41f5865e5b9ab6e71b5e1ac1"
    end
  end

  def install
    bin.install Dir["*"].first => "adot"
  end

  test do
    assert_match "adot", shell_output("#{bin}/adot --help")
  end
end

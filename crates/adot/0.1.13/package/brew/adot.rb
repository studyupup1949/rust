class Adot < Formula
  desc "A minimal dotfile manager"
  homepage "https://github.com/Dimfred/adot"
  version "0.1.12"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Dimfred/adot/releases/download/v0.1.12/macos-adot-v0.1.12-arm64"
      sha256 "149b36b3aa01d5a42b10622976d5e241bba249cfb9c0df1fe1e2bf36a166a557"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Dimfred/adot/releases/download/v0.1.12/linux-adot-v0.1.12-x86_64"
      sha256 "323721c5dd58612e4e53b7007fb6766b769c85d9159ee5cf1ab92da7d7406919"
    end
  end

  def install
    bin.install Dir["*"].first => "adot"
  end

  test do
    assert_match "adot", shell_output("#{bin}/adot --help")
  end
end

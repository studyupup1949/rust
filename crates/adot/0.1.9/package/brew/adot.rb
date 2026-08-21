class Adot < Formula
  desc "A minimal dotfile manager"
  homepage "https://github.com/Dimfred/adot"
  version "0.1.8"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Dimfred/adot/releases/download/v0.1.8/macos-adot-v0.1.8-arm64"
      sha256 "5c053c240f5a60d05d9fca00209cb862f6a3ee2b7f42131ace80d909ac4fe78d"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Dimfred/adot/releases/download/v0.1.8/linux-adot-v0.1.8-x86_64"
      sha256 "8a206a167c6165a4aafbe506de9c422555fae9b83a263724b12d5b76cd842009"
    end
  end

  def install
    bin.install Dir["*"].first => "adot"
  end

  test do
    assert_match "adot", shell_output("#{bin}/adot --help")
  end
end

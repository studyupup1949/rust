class Adot < Formula
  desc "A minimal dotfile manager"
  homepage "https://github.com/Dimfred/adot"
  version "0.1.9"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Dimfred/adot/releases/download/v0.1.9/macos-adot-v0.1.9-arm64"
      sha256 "f698880e18c968e9cf224ec0fe7640ced2d2852879da9802fef414e605e0cb5a"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Dimfred/adot/releases/download/v0.1.9/linux-adot-v0.1.9-x86_64"
      sha256 "664f74022d2f8e73c625fc44a026ff052c42244732c3cf97d8c1ad91c111d249"
    end
  end

  def install
    bin.install Dir["*"].first => "adot"
  end

  test do
    assert_match "adot", shell_output("#{bin}/adot --help")
  end
end

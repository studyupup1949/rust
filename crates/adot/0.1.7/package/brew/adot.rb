class Adot < Formula
  desc "A minimal dotfile manager"
  homepage "https://github.com/Dimfred/adot"
  version "0.1.6"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Dimfred/adot/releases/download/v0.1.6/macos-adot-v0.1.6-arm64"
      sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Dimfred/adot/releases/download/v0.1.6/linux-adot-v0.1.6-x86_64"
      sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    end
  end

  def install
    bin.install Dir["*"].first => "adot"
  end

  test do
    assert_match "adot", shell_output("#{bin}/adot --help")
  end
end

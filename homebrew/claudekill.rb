class Claudekill < Formula
  desc "A high-velocity, terminal-native instrument designed to hunt down and neutralize .claude cache directories"
  homepage "https://github.com/olbboy/claudekill"
  version "0.1.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/olbboy/claudekill/releases/download/v#{version}/claudekill-aarch64-apple-darwin.tar.gz"
      sha256 "02512517ce5b880b0a1afef7e4af74993a4e269354c999967b5128aa5f7b1caf"
    end
    on_intel do
      url "https://github.com/olbboy/claudekill/releases/download/v#{version}/claudekill-x86_64-apple-darwin.tar.gz"
      sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    end
  end

  def install
    bin.install "claudekill"
  end

  test do
    assert_match "claudekill", shell_output("#{bin}/claudekill --version")
  end
end

class Claudekill < Formula
  desc "Find and delete .claude folders to reclaim disk space"
  homepage "https://github.com/olbboy/claudekill"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/olbboy/claudekill/releases/download/v#{version}/claudekill-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    end
    on_intel do
      url "https://github.com/olbboy/claudekill/releases/download/v#{version}/claudekill-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X64"
    end
  end

  def install
    bin.install "claudekill"
  end

  test do
    assert_match "claudekill", shell_output("#{bin}/claudekill --version")
  end
end

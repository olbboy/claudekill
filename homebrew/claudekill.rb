class Claudekill < Formula
  desc "A high-velocity, terminal-native instrument designed to hunt down and neutralize .claude cache directories"
  homepage "https://github.com/olbboy/claudekill"
  version "0.5.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/olbboy/claudekill/releases/download/v#{version}/claudekill-aarch64-apple-darwin.tar.gz"
      sha256 "cce510b2de683390bef7d0cbf163fec6e4172425346b70c3119235c052ace338"
    end
    on_intel do
      url "https://github.com/olbboy/claudekill/releases/download/v#{version}/claudekill-x86_64-apple-darwin.tar.gz"
      sha256 "abc07c6554131244086f35fabf9e7db5e15e0c834b4da8ba26df8f9c93136039"
    end
  end

  def install
    bin.install "claudekill"
  end

  test do
    assert_match "claudekill", shell_output("#{bin}/claudekill --version")
  end
end

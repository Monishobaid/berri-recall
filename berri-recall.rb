class BerriRecall < Formula
  desc "Your terminal remembers what you typed last week"
  homepage "https://github.com/monishobaid/berri-recall"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/monishobaid/berri-recall/releases/download/v#{version}/berri-recall-macos-arm64.tar.gz"
      sha256 "" # Add after first release
    else
      url "https://github.com/monishobaid/berri-recall/releases/download/v#{version}/berri-recall-macos-amd64.tar.gz"
      sha256 "" # Add after first release
    end
  end

  on_linux do
    url "https://github.com/monishobaid/berri-recall/releases/download/v#{version}/berri-recall-linux-amd64.tar.gz"
    sha256 "" # Add after first release
  end

  def install
    bin.install "berri-recall"
  end

  def caveats
    <<~EOS
      Run 'berri-recall setup' to install shell hooks and start recording commands automatically.

      Your commands will be stored locally in ~/.berri-recall/
    EOS
  end

  test do
    assert_match "berri-recall v#{version}", shell_output("#{bin}/berri-recall --version")
  end
end

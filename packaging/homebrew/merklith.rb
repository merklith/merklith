class Merklith < Formula
  desc "MERKLITH Blockchain - Where Trust is Forged"
  homepage "https://merklith.com"
  url "https://github.com/merklith/merklith/archive/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license any_of: ["Apache-2.0", "MIT"]
  head "https://github.com/merklith/merklith.git", branch: "main"

  depends_on "rust" => :build
  depends_on "pkg-config" => :build

  def install
    system "cargo", "build", "--release"
    
    bin.install "target/release/merklith"
    bin.install "target/release/merklith-node"
    bin.install "target/release/merklith-monitor"
    bin.install "target/release/merklith-benchmark"
    bin.install "target/release/merklith-faucet"
    
    # Install completions
    bash_completion.install "completions/merklith.bash" => "merklith"
    zsh_completion.install "completions/_merklith"
    fish_completion.install "completions/merklith.fish"
    
    # Install documentation
    doc.install Dir["docs/*"]
    doc.install "README.md"
    doc.install "CHANGELOG.md"
    doc.install "LICENSE-APACHE"
    doc.install "LICENSE-MIT"
  end

  def post_install
    (var/"merklith").mkpath
    (etc/"merklith").mkpath
  end

  service do
    run [opt_bin/"merklith-node", "--config", etc/"merklith/config.toml"]
    keep_alive true
    log_path var/"log/merklith.log"
    error_log_path var/"log/merklith.error.log"
  end

  test do
    system "#{bin}/merklith", "--version"
    system "#{bin}/merklith-node", "--version"
  end
end
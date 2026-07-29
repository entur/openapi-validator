class Lazyoav < Formula
  desc "Interactive TUI for linting, generating, and compiling OpenAPI specs"
  homepage "https://github.com/entur/openapi-validator"
  version "0.1.1"
  license "EUPL-1.2"

  # version and sha256 values are updated automatically by .github/workflows/release.yml on each tui release.

  on_macos do
    on_intel do
      url "https://github.com/entur/openapi-validator/releases/download/tui-v#{version}/lazyoav-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "3b6d6be55ea854e54106cdec5969d1c9dfbeffb41005d99b21c0028254639875"
    end

    on_arm do
      url "https://github.com/entur/openapi-validator/releases/download/tui-v#{version}/lazyoav-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "00ca005788b68717b5489de02b9593d8e4d1204d18f0ede4bab934714f58f839"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/entur/openapi-validator/releases/download/tui-v#{version}/lazyoav-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "931322477088866cf539cc6a125495d05ca3d260c4375014477bb87748e4fdb6"
    end
  end

  def install
    bin.install "lazyoav"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/lazyoav --version")
  end
end

class Lazyoav < Formula
  desc "Interactive TUI for linting, generating, and compiling OpenAPI specs"
  homepage "https://github.com/entur/openapi-validator"
  version "0.1.0"
  license "EUPL-1.2"

  # version and sha256 values are updated automatically by .github/workflows/release.yml on each tui release.

  on_macos do
    on_intel do
      url "https://github.com/entur/openapi-validator/releases/download/tui-v#{version}/lazyoav-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "28bea8138196171a0aa588904a0def054fe09c1114eddecff935912938198382"
    end

    on_arm do
      url "https://github.com/entur/openapi-validator/releases/download/tui-v#{version}/lazyoav-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "dd3ed40be6facd21becb883c38e118ab6a4a18b52c6dccc15d26ade61f2cee86"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/entur/openapi-validator/releases/download/tui-v#{version}/lazyoav-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "eda6cad4518b5af00c926c4d24d1800c23258e5e1963806265d27f400234b161"
    end
  end

  def install
    bin.install "lazyoav"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/lazyoav --version")
  end
end

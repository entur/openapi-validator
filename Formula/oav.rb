class Oav < Formula
  desc "OpenAPI Validator CLI for linting, generating, and compiling OpenAPI specs locally"
  homepage "https://github.com/entur/openapi-validator"
  version "0.7.2"
  license "EUPL-1.2"

  # version and sha256 values are updated automatically by .github/workflows/release.yml on each cli release.

  on_macos do
    on_intel do
      url "https://github.com/entur/openapi-validator/releases/download/cli-v#{version}/oav-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "a9652ddd8ebfac50b682d13d32d543659c8e751de424d6b6a1e39e22544c6fe4"
    end

    on_arm do
      url "https://github.com/entur/openapi-validator/releases/download/cli-v#{version}/oav-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "5d5b31d1b1165478149fb679377df067570d43ea7829f4cf5049ed6fb107c204"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/entur/openapi-validator/releases/download/cli-v#{version}/oav-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "fd57b80b63d10bde3b13544b70c62cddcbe28c280b25f20fca14e5107cfa5427"
    end
  end

  def install
    bin.install "oav"
    generate_completions_from_executable(bin/"oav", "completions", "generate")
  end

  test do
    assert_match "OpenAPI Validator", shell_output("#{bin}/oav --help")
  end
end

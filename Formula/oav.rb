class Oav < Formula
  desc "OpenAPI Validator CLI for linting, generating, and compiling OpenAPI specs locally"
  homepage "https://github.com/entur/openapi-validator"
  version "0.7.0"
  license "EUPL-1.2"

  # version and sha256 values are updated automatically by .github/workflows/release.yml on each cli release.

  on_macos do
    on_intel do
      url "https://github.com/entur/openapi-validator/releases/download/cli-v#{version}/oav-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end

    on_arm do
      url "https://github.com/entur/openapi-validator/releases/download/cli-v#{version}/oav-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/entur/openapi-validator/releases/download/cli-v#{version}/oav-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
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

class Oav < Formula
  desc "OpenAPI Validator CLI for linting, generating, and compiling OpenAPI specs locally"
  homepage "https://github.com/entur/openapi-validator"
  version "0.7.1"
  license "EUPL-1.2"

  # version and sha256 values are updated automatically by .github/workflows/release.yml on each cli release.

  on_macos do
    on_intel do
      url "https://github.com/entur/openapi-validator/releases/download/cli-v#{version}/oav-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "610e0167bfd03b95032032e47d354a1ecb54c0d387a148e88013631f6b7a5c5f"
    end

    on_arm do
      url "https://github.com/entur/openapi-validator/releases/download/cli-v#{version}/oav-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "0986f0a5fd921edfa2603be930ef9334d19cbf34b774b6667e3324e5d7e1778d"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/entur/openapi-validator/releases/download/cli-v#{version}/oav-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "f5b7e6a6909e379d13cfcc73880b30c1219185da5166189e71573fdf97507835"
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

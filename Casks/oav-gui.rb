cask "oav-gui" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.1.0"
  sha256 arm:   "1091c36dc9bc64a9bd113f14c5024f42da31fde48e0a819541fbcc2acd7b6c47",
         intel: "3d1f36d4d8e77681cc78a68b0ce9fd07aa1fcf7420745827ec7ab89fedc0e4dc"

  # version and sha256 values are updated automatically by .github/workflows/release.yml on each gui release.

  url "https://github.com/entur/openapi-validator/releases/download/gui-v#{version}/oav-gui-#{version}-#{arch}-apple-darwin.dmg"
  name "OpenAPI Validator"
  desc "Desktop GUI for linting, validating, and editing OpenAPI specs"
  homepage "https://github.com/entur/openapi-validator"

  depends_on :macos

  app "OpenAPI Validator.app"

  zap trash: [
    "~/Library/Application Support/org.entur.oav",
    "~/Library/Caches/org.entur.oav",
    "~/Library/Preferences/org.entur.oav.plist",
    "~/Library/Saved Application State/org.entur.oav.savedState",
    "~/Library/WebKit/org.entur.oav",
  ]

  caveats <<~EOS
    The app is not signed or notarized, so macOS quarantines it on first
    launch. Either install with:

      brew install --cask --no-quarantine oav-gui

    or right-click the app in Finder and choose Open the first time.
  EOS
end

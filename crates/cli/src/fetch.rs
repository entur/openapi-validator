use anyhow::Result;
use indicatif::ProgressBar;
use std::path::{Path, PathBuf};

use crate::output::Output;

pub use oav_lib::fetch::looks_like_url;

/// Guard that clears a spinner on drop, ensuring it never leaks on early returns.
struct SpinnerGuard<'a> {
    spinner: Option<ProgressBar>,
    output: &'a Output,
    label: &'static str,
    success: bool,
}

impl<'a> SpinnerGuard<'a> {
    fn finish(&mut self, label: &'static str, success: bool) {
        self.label = label;
        self.success = success;
    }
}

impl Drop for SpinnerGuard<'_> {
    fn drop(&mut self) {
        self.output
            .finish_spinner(self.spinner.as_ref(), self.label, self.success);
    }
}

/// Fetch a spec from a URL and write it to `.oav/fetched-spec.{ext}`.
/// Returns the path to the fetched file, relative to `root`.
pub fn fetch_spec(root: &Path, url: &str, output: &Output) -> Result<PathBuf> {
    oav_lib::fetch::validate_url(url)?;

    let mut guard = SpinnerGuard {
        spinner: output.start_spinner(&format!("Fetching spec from {url}")),
        output,
        label: "Fetch failed",
        success: false,
    };

    let relative = oav_lib::fetch::fetch_spec(root, url)?;
    guard.finish("Fetched spec", true);
    Ok(relative)
}

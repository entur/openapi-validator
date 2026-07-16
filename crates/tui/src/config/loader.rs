use super::types::Config;
use crate::custom::CustomGeneratorDef;

pub use oav_lib::config::load;

/// Validate config against the built-in and custom generator registries.
///
/// Returns warning messages for unknown generators and stale overrides.
/// These are warnings, not errors — unknown generators still run via bare
/// `-g`. Structural errors (bad timeout, depth, or job count) also surface
/// as a warning here; the TUI stays up and shows them in the status bar.
pub fn validate(cfg: &Config, custom_defs: &[CustomGeneratorDef]) -> Vec<String> {
    match oav_lib::config::validate_for_run(cfg, custom_defs) {
        Ok(warnings) => warnings,
        Err(e) => vec![e.to_string()],
    }
}

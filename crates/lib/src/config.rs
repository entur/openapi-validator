use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const CONFIG_FILE: &str = ".oavc";

#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Server,
    Client,
    Both,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Server => "server",
            Mode::Client => "client",
            Mode::Both => "both",
        }
    }
}

#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Linter {
    Spectral,
    Redocly,
    None,
}

impl Linter {
    pub fn as_str(&self) -> &'static str {
        match self {
            Linter::Spectral => "spectral",
            Linter::Redocly => "redocly",
            Linter::None => "none",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jobs {
    Auto,
    Fixed(usize),
}

impl Jobs {
    pub fn resolve(self) -> usize {
        match self {
            Jobs::Fixed(n) => n,
            Jobs::Auto => std::thread::available_parallelism()
                .map(|n| n.get().min(4))
                .unwrap_or(1),
        }
    }
}

impl Serialize for Jobs {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Jobs::Auto => serializer.serialize_str("auto"),
            Jobs::Fixed(n) => serializer.serialize_u64(*n as u64),
        }
    }
}

impl<'de> Deserialize<'de> for Jobs {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct JobsVisitor;

        impl<'de> Visitor<'de> for JobsVisitor {
            type Value = Jobs;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("\"auto\" or a positive integer")
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Jobs, E> {
                if value == 0 {
                    return Err(E::custom("jobs must be \"auto\" or a positive integer"));
                }
                Ok(Jobs::Fixed(value as usize))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Jobs, E> {
                if value <= 0 {
                    return Err(E::custom("jobs must be \"auto\" or a positive integer"));
                }
                Ok(Jobs::Fixed(value as usize))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Jobs, E> {
                if value.eq_ignore_ascii_case("auto") {
                    Ok(Jobs::Auto)
                } else {
                    Err(E::custom("jobs must be \"auto\" or a positive integer"))
                }
            }
        }

        deserializer.deserialize_any(JobsVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub spec: Option<String>,
    pub mode: Mode,
    pub lint: bool,
    pub generate: bool,
    pub compile: bool,
    pub server_generators: Vec<String>,
    pub client_generators: Vec<String>,
    pub generator_overrides: HashMap<String, String>,
    pub generator_image: String,
    pub redocly_image: String,
    pub linter: Linter,
    pub spectral_image: String,
    pub spectral_ruleset: String,
    pub spectral_fail_severity: String,
    pub manage_gitignore: bool,
    pub custom_generators_dir: Option<String>,
    pub docker_timeout: u64,
    pub search_depth: usize,
    pub jobs: Jobs,
    /// TUI-only: action → key bindings. CLI ignores this field; omitted from
    /// serialized output when empty so CLI-only `.oavc` files don't grow a
    /// stray `keys: {}` entry.
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        deserialize_with = "deserialize_keys"
    )]
    pub keys: HashMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            spec: None,
            mode: Mode::Server,
            lint: true,
            generate: true,
            compile: true,
            server_generators: Vec::new(),
            client_generators: Vec::new(),
            generator_overrides: HashMap::new(),
            generator_image: "openapitools/openapi-generator-cli:v7.17.0".to_string(),
            redocly_image: "redocly/cli:1.25.5".to_string(),
            linter: Linter::Spectral,
            spectral_image: "stoplight/spectral:6".to_string(),
            spectral_ruleset:
                "https://raw.githubusercontent.com/entur/api-guidelines/refs/tags/v2/.spectral.yml"
                    .to_string(),
            spectral_fail_severity: "error".to_string(),
            manage_gitignore: true,
            custom_generators_dir: None,
            docker_timeout: 300,
            search_depth: 4,
            jobs: Jobs::Auto,
            keys: HashMap::new(),
        }
    }
}

/// Load config from `.oavc` in the given directory.
/// Returns the default config if the file doesn't exist.
pub fn load(root: &Path) -> Result<Config> {
    let path = root.join(CONFIG_FILE);
    if !path.exists() {
        return Ok(Config::default());
    }
    if !path.is_file() {
        anyhow::bail!(".oavc exists but is not a file: {}", path.display());
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let config: Config = yaml_serde::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(config)
}

/// Write config as YAML to `.oavc` in the given directory.
pub fn write(root: &Path, config: &Config) -> Result<()> {
    let path = root.join(CONFIG_FILE);
    let content = yaml_serde::to_string(config).context("Failed to serialize config")?;
    fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Accept both scalar strings and lists per action in the `keys` config map.
///
/// Lets users write either form in `.oavc`:
/// ```yaml
/// keys:
///   scroll_down: "j"          # single string
///   quit: ["q", "C-c"]        # list of strings
///   toggle_diff: []            # explicit unbind
/// ```
fn deserialize_keys<'de, D>(deserializer: D) -> Result<HashMap<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct KeysVisitor;

    impl<'de> Visitor<'de> for KeysVisitor {
        type Value = HashMap<String, Vec<String>>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a map of action names to key strings or lists of key strings")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let mut result = HashMap::new();
            while let Some(key) = map.next_key::<String>()? {
                let value: StringOrVec = map.next_value()?;
                result.insert(key, value.into_vec());
            }
            Ok(result)
        }
    }

    deserializer.deserialize_map(KeysVisitor)
}

#[derive(Debug)]
enum StringOrVec {
    Single(String),
    Multiple(Vec<String>),
}

impl StringOrVec {
    fn into_vec(self) -> Vec<String> {
        match self {
            StringOrVec::Single(s) => vec![s],
            StringOrVec::Multiple(v) => v,
        }
    }
}

impl<'de> Deserialize<'de> for StringOrVec {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StringOrVecVisitor;

        impl<'de> Visitor<'de> for StringOrVecVisitor {
            type Value = StringOrVec;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string or a list of strings")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<StringOrVec, E> {
                Ok(StringOrVec::Single(value.to_owned()))
            }

            fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<StringOrVec, S::Error> {
                let mut v = Vec::new();
                while let Some(s) = seq.next_element::<String>()? {
                    v.push(s);
                }
                Ok(StringOrVec::Multiple(v))
            }
        }

        deserializer.deserialize_any(StringOrVecVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_config(yaml: &str) -> Config {
        yaml_serde::from_str(yaml).expect("should parse")
    }

    #[test]
    fn keys_scalar_string_wraps_into_vec() {
        let cfg = parse_config("keys:\n  scroll_down: \"j\"\n");
        assert_eq!(cfg.keys["scroll_down"], vec!["j"]);
    }

    #[test]
    fn keys_list_survives_custom_deserializer() {
        let cfg = parse_config("keys:\n  quit: [\"q\", \"C-c\"]\n");
        assert_eq!(cfg.keys["quit"], vec!["q", "C-c"]);
    }

    #[test]
    fn keys_empty_list_unbinds() {
        let cfg = parse_config("keys:\n  toggle_diff: []\n");
        assert!(cfg.keys["toggle_diff"].is_empty());
    }

    #[test]
    fn keys_bare_y_n_are_strings_not_booleans() {
        // YAML 1.2: y/n/yes/no are strings, not booleans.
        // Guard against a yaml lib upgrade silently breaking single-char bindings.
        let cfg = parse_config("keys:\n  scroll_down: y\n  scroll_up: n\n");
        assert_eq!(cfg.keys["scroll_down"], vec!["y"]);
        assert_eq!(cfg.keys["scroll_up"], vec!["n"]);
    }

    #[test]
    fn keys_integer_value_is_rejected() {
        let result = yaml_serde::from_str::<Config>("keys:\n  scroll_down: 42\n");
        assert!(result.is_err());
    }

    #[test]
    fn empty_keys_is_omitted_from_serialized_output() {
        let cfg = Config::default();
        let yaml = yaml_serde::to_string(&cfg).expect("should serialize");
        assert!(
            !yaml.contains("keys:"),
            "default config should not emit a `keys:` line, got:\n{yaml}"
        );
    }

    #[test]
    fn populated_keys_is_serialized() {
        let mut cfg = Config::default();
        cfg.keys.insert("scroll_down".into(), vec!["j".into()]);
        let yaml = yaml_serde::to_string(&cfg).expect("should serialize");
        assert!(yaml.contains("keys:"), "expected `keys:` in:\n{yaml}");
    }
}

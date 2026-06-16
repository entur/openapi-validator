use std::fmt;

use serde::de::{self, Visitor};
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

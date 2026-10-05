//! YAML config loader (figment + serde_yaml).
//!
//! Used to load `config/compliance/ccfg-*.yaml` compliance configurations and
//! any other versioned YAML configs.

use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum YamlConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml parse: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("validation: {0}")]
    Validation(String),
}

/// Load a YAML config from a file path.
pub fn load_yaml_config<T: DeserializeOwned + Serialize>(
    path: impl AsRef<Path>,
) -> Result<T, YamlConfigError> {
    let content = std::fs::read_to_string(path)?;
    let parsed: T = serde_yaml::from_str(&content)?;
    Ok(parsed)
}

/// Load a YAML config from a string.
pub fn load_yaml_config_from_str<T: DeserializeOwned>(s: &str) -> Result<T, YamlConfigError> {
    let parsed: T = serde_yaml::from_str(s)?;
    Ok(parsed)
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Cfg {
        name: String,
        count: u32,
    }

    #[test]
    fn parse_yaml_string() {
        let s = "name: test\ncount: 42\n";
        let cfg: Cfg = load_yaml_config_from_str(s).unwrap();
        assert_eq!(cfg.name, "test");
        assert_eq!(cfg.count, 42);
    }
}

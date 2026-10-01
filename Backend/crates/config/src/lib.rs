//! `lcc-config` — typed env + YAML config loader.
//!
//! Used by every service at startup. The YAML loader powers compliance
//! config version loading.

pub mod env;
pub mod yaml;

pub use env::{load_env_config, EnvConfig, Environment};
pub use yaml::{load_yaml_config, YamlConfigError};

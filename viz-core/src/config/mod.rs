//! Application Configuration

use std::{env::current_dir, fs, path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use toml::{
    self,
    value::{Map, Value},
};

use viz_utils::{futures::future::BoxFuture, tracing};

use crate::{Context, Error, Extract, Result};

mod cookies;
mod env;
mod limits;

pub use cookies::Cookies;
pub use env::Env;
pub use limits::Limits;

/// Config
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Env
    #[serde(skip_deserializing)]
    pub env: Env,

    /// Limits
    #[serde(default)]
    pub limits: Limits,

    /// Cookies
    #[serde(default)]
    pub cookies: Cookies,

    /// Extras
    #[serde(default)]
    pub extras: Map<String, Value>,

    /// Dir
    #[serde(skip_deserializing)]
    pub dir: PathBuf,
}

impl Config {
    /// Loads config file
    pub async fn load() -> Result<Config> {
        let path = current_dir()?;

        let e = Env::get();

        let config_path = path.join("config").join(e.to_string() + ".toml");

        let mut config = if config_path.exists() {
            toml::from_str(&fs::read_to_string(config_path)?).unwrap_or_default()
        } else {
            Config::default()
        };

        config.dir = path;
        config.env = e;

        tracing::info!("{:#?}", config);

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            cookies: Cookies::default(),
            env: Env::default(),
            extras: Map::default(),
            dir: current_dir().unwrap_or_default(),
        }
    }
}

impl Config {
    /// Creates new Config instance
    pub fn new() -> Self {
        Self::default()
    }
}

impl Extract for Arc<Config> {
    type Error = Error;

    #[inline]
    fn extract(cx: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>> {
        Box::pin(async move { Ok(cx.config()) })
    }
}

/// Extends Context
impl Context {
    /// Gets application config
    pub fn config(&self) -> Arc<Config> {
        self.extensions().get::<Arc<Config>>().cloned().unwrap_or_default()
    }
}

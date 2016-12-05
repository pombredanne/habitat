// Copyright (c) 2016 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use hcore::config::{ConfigFile, ParseInto};
use hcore::env as henv;
use hcore::fs::{am_i_root, FS_ROOT_PATH};
use hcore::os::users;
use toml;

use error::{Error, Result};

const CLI_CONFIG_PATH: &'static str = "hab/etc/cli.toml";

pub fn load() -> Result<Config> {
    common_load(false)
}

pub fn load_with_sudo_user() -> Result<Config> {
    common_load(true)
}

pub fn save(config: &Config) -> Result<()> {
    let config_path = cli_config_path(false);
    let parent_path = match config_path.parent() {
        Some(p) => p,
        None => return Err(Error::FileNotFound(config_path.to_string_lossy().into_owned())),
    };
    try!(fs::create_dir_all(&parent_path));
    let raw = toml::encode_str(config);
    debug!("Raw config toml:\n---\n{}\n---", &raw);
    let mut file = try!(File::create(&config_path));
    try!(file.write_all(raw.as_bytes()));
    Ok(())
}

fn common_load(use_sudo_user: bool) -> Result<Config> {
    let cli_config_path = cli_config_path(use_sudo_user);
    if cli_config_path.exists() {
        debug!("Loading CLI config from {}", cli_config_path.display());
        Ok(try!(Config::from_file(&cli_config_path)))
    } else {
        debug!("No CLI config found, loading defaults");
        Ok(Config::default())
    }
}

fn cli_config_path(use_sudo_user: bool) -> PathBuf {
    match am_i_root() {
        true => {
            if use_sudo_user {
                if let Some(sudo_user) = henv::sudo_user() {
                    if let Some(home) = users::get_home_for_user(&sudo_user) {
                        return home.join(format!(".{}", CLI_CONFIG_PATH));
                    }
                }
            }
        }
        false => {
            if let Some(home) = env::home_dir() {
                return home.join(format!(".{}", CLI_CONFIG_PATH));
            }
        }
    }

    PathBuf::from(FS_ROOT_PATH).join(CLI_CONFIG_PATH)
}

#[derive(Clone, Debug, RustcEncodable)]
enum ConfigError {
    MissingConfigurationValue,
}

#[derive(Clone, Debug, RustcEncodable)]
struct ConfigValue {
    config_file: Option<String>,
    environment: Option<String>,
    cli_parameter: Option<String>,
}

impl ConfigValue {
    fn get(self) -> Result<String, ConfigError> {
        match self {
            ConfigValue { cli_parameter: Some(value), .. } => Ok(value),
            ConfigValue { environment: Some(value), .. } => Ok(value),
            ConfigValue { config_file: Some(value), .. } => Ok(value),
            _ => Err(ConfigError::MissingConfigurationValue),
        }
    }
}

impl Default for ConfigValue {
    fn default() -> Self {
        ConfigValue {
            cli_parameter: None,
            environment: None,
            config_file: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, RustcEncodable)]
pub struct Config {
    pub auth_token: ConfigValue,
    pub origin: ConfigValue,
    pub depot_url: ConfigValue,
}

impl ConfigFile for Config {
    type Error = Error;

    fn from_toml(toml: toml::Value) -> Result<Self> {
        let mut cfg = Config::default();
        try!(toml.parse_into("auth_token", &mut cfg.auth_token.config_file));
        try!(toml.parse_into("origin", &mut cfg.origin.config_file));
        try!(toml.parse_into("depot_url", &mut cfg.depot_url.config_file));
        Ok(cfg)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auth_token: ConfigValue::default(),
            origin: ConfigValue::default(),
            depot_url: ConfigValue::default(),
        }
    }
}

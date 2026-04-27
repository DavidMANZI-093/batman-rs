use std::{fs, process::Command};

use crate::{core::Config, error, info, warn};

pub fn load_config(path: &str) -> Option<Config> {
    let config_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            info!("No config found at {}, falling back to defaults.", path);
            return None;
        }
    };

    match toml::from_str::<Config>(&config_content) {
        Ok(config) => {
            for rule in &config.rules {
                if let Some(binary) = rule.command.split_whitespace().next()
                    && !command_exists(binary)
                {
                    warn!(
                        "Binary '{}' not found in PATH. This rule will fail when triggered!",
                        binary
                    );
                }
            }

            info!("Successfully loaded {} rules.", config.rules.len());
            Some(config)
        }
        Err(e) => {
            error!("Syntax error in config file: {}", e);
            std::process::exit(1);
        }
    }
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

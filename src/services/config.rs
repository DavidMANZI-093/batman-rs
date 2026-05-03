use std::{fs, process::Command};

use crate::{core::Config, debug, error, info, warn};

pub fn load_config(path: &str) -> Config {
    let config_content = fs::read_to_string(path).unwrap_or_else(|e| {
        error!("Failed to read config file '{}': {}", path, e);
        std::process::exit(1);
    });

    let config: Config = toml::from_str(&config_content).unwrap_or_else(|e| {
        error!("Syntax error in config file: {}", e);
        std::process::exit(1);
    });

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
    config
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|out| out.status.success())
        .unwrap_or_else(|e| {
            debug!("Failed to run 'which {}': {}", cmd, e);
            false
        })
}

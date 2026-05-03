use crate::utils::cli::Cli;
use crate::{debug, info};
use std::env;
use std::path::Path;

pub fn find_config_path(cli_args: &Cli) -> Result<String, String> {
    if let Some(path) = &cli_args.config {
        if Path::new(path).exists() {
            info!("Using CLI-provided config: {}", path);
            return Ok(path.clone());
        } else {
            return Err(format!(
                "CLI config path provided but file does not exist: {}",
                path
            ));
        }
    }

    let user_config = if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        format!("{}/batman/config.toml", xdg)
    } else if let Ok(home) = env::var("HOME") {
        format!("{}/.config/batman/config.toml", home)
    } else {
        String::new()
    };

    if !user_config.is_empty() && Path::new(&user_config).exists() {
        debug!("Found user config at: {}", user_config);
        return Ok(user_config);
    }

    let system_config = "/etc/batman/config.toml";
    if Path::new(system_config).exists() {
        debug!("Found system config at: {}", system_config);
        return Ok(system_config.to_string());
    }

    Err("No config file found. Checked CLI path, user config (~/.config/batman/config.toml), and system config (/etc/batman/config.toml).".to_string())
}

use std::process::Command;

use crate::{error, info, warn};

pub fn execute_rule(command_string: &str, is_critical: bool) {
    let mut parts = command_string.split_whitespace();

    if let Some(binary) = parts.next() {
        let binary_owned = binary.to_string();
        let args: Vec<String> = parts.map(|s| s.to_string()).collect();

        match Command::new(&binary_owned).args(&args).spawn() {
            Ok(mut child) => {
                info!("Executed rule: {}", command_string);

                std::thread::spawn(move || {
                    if let Ok(status) = child.wait()
                        && !status.success()
                    {
                        warn!(
                            "Command '{}' exited with an error code.",
                            binary_owned.clone()
                        );

                        if is_critical {
                            emergency_shutdown();
                        }
                    }
                });
            }
            Err(e) => {
                error!(
                    "CRITICAL: Failed to spawn command '{}'. Error: {}",
                    binary, e
                );

                if is_critical {
                    emergency_shutdown();
                }
            }
        }
    }
}

fn emergency_shutdown() {
    error!("Executing fallback to protect hardware!");

    if Command::new("systemctl").arg("poweroff").spawn().is_ok() {
        return;
    }

    if Command::new("loginctl").arg("poweroff").spawn().is_ok() {
        return;
    }

    if Command::new("shutdown").args(["-h", "now"]).spawn().is_ok() {
        return;
    }

    error!("ALL FALLBACKS FAILED. System is going to crash from power loss.");
}

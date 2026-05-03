use std::process::Command;

use crate::{
    core::{
        BatteryStatus, Config, PowerEvent,
        models::{RuleState, SystemPowerState},
    },
    error, info, warn,
};

pub fn process_event(event: &PowerEvent, current_state: &mut SystemPowerState, config: &Config) {
    let old_state = current_state.clone();

    match event {
        PowerEvent::Battery { capacity, status } => {
            current_state.capacity = *capacity;
            current_state.status = status.clone();
        }
        PowerEvent::AcAdapter { online } => {
            current_state.ac_online = *online;
        }
    }

    for rule in &config.rules {
        let scenario_matches = match rule.state {
            RuleState::Charging => current_state.status == BatteryStatus::Charging,
            RuleState::Discharging => current_state.status == BatteryStatus::Discharging,
            RuleState::AcOnline => current_state.ac_online == true,
            RuleState::AcOffline => current_state.ac_online == false,
            _ => false,
        };

        if !scenario_matches {
            continue;
        }

        let threshold_met = match rule.capacity_under {
            Some(threshold) => {
                let was_safely_above = old_state.capacity > threshold;
                let is_now_below_or_equal = current_state.capacity <= threshold;

                was_safely_above && is_now_below_or_equal
            }
            None => true,
        };

        if scenario_matches && threshold_met {
            execute_rule(&rule.command, current_state.capacity <= 5);
        }
    }
}

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

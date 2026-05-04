use std::process::Command;

use crate::{
    core::{
        BatteryStatus, Config, PowerEvent,
        models::{RuleState, SystemPowerState},
    },
    error, info, warn,
};

pub fn process_event(event: &PowerEvent, current_state: &mut SystemPowerState, config: &Config) {
    let old_state: SystemPowerState = current_state.clone();

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
        let scenario_matches: bool = match rule.state {
            RuleState::Charging => current_state.status == BatteryStatus::Charging,
            RuleState::Discharging => current_state.status == BatteryStatus::Discharging,
            RuleState::Full => current_state.status == BatteryStatus::Full,
            RuleState::NotCharging => current_state.status == BatteryStatus::NotCharging,
            RuleState::AcOnline => current_state.ac_online,
            RuleState::AcOffline => !current_state.ac_online,
        };

        if !scenario_matches {
            continue;
        }

        let threshold_met: bool = match rule.capacity_under {
            Some(threshold) => {
                let was_safely_above = old_state.capacity > threshold;
                let is_now_below_or_equal = current_state.capacity <= threshold;

                was_safely_above && is_now_below_or_equal
            }
            None => match rule.state {
                RuleState::Charging    => old_state.status != BatteryStatus::Charging,
                RuleState::Discharging => old_state.status != BatteryStatus::Discharging,
                RuleState::Full        => old_state.status != BatteryStatus::Full,
                RuleState::NotCharging => old_state.status != BatteryStatus::NotCharging,
                RuleState::AcOnline    => !old_state.ac_online,
                RuleState::AcOffline   => old_state.ac_online,
            },
        };

        if threshold_met {
            execute_rule(
                &rule.command,
                current_state.capacity <= 5 && current_state.status == BatteryStatus::Discharging,
            );
        }
    }
}

fn execute_rule(command_string: &str, is_critical: bool) {
    let command_string_owned = command_string.to_string();

    match Command::new("sh").args(["-c", command_string]).spawn() {
        Ok(mut child) => {
            info!("Executed rule: {}", command_string);

            std::thread::spawn(move || {
                if let Ok(status) = child.wait()
                    && !status.success()
                {
                    warn!(
                        "Command '{}' exited with an error code.",
                        command_string_owned
                    );

                    if is_critical {
                        emergency_shutdown();
                    }
                }
            });
        }
        Err(e) => {
            error!(
                "Critical: Failed to spawn command '{}'. Error: {}",
                command_string, e
            );

            if is_critical {
                emergency_shutdown();
            }
        }
    }
}

fn emergency_shutdown() {
    error!("Executing fallback to protect hardware!");

    if Command::new("systemctl").arg("poweroff").spawn().is_ok() {
        return;
    }
    warn!("Systemctl poweroff failed, trying loginctl...");

    if Command::new("loginctl").arg("poweroff").spawn().is_ok() {
        return;
    }
    warn!("Loginctl poweroff failed, trying shutdown...");

    if Command::new("shutdown").args(["-h", "now"]).spawn().is_ok() {
        return;
    }
    warn!("Shutdown -h now failed, no more fallbacks!");

    error!("All fallbacks failed. System is going to crash from power loss.");
}

use std::fs;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

#[derive(Debug)]
pub enum PowerEvent {
    Battery { capacity: u8, status: BatteryStatus },
    AcAdapter { online: bool },
}

#[derive(Debug)]
pub enum ParseError {
    MissingFields,
    InvalidUtf8,
    InvalidCapacity(String),
    InvalidStatus(String),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub state: RuleState,
    pub capacity_under: Option<u8>,
    pub command: String,
}

#[derive(Debug, Deserialize)]
pub enum RuleState {
    Charging,
    Discharging,
    Full,
    NotCharging,
    AcOnline,
    AcOffline,
}

#[derive(Debug, Clone)]
pub struct SystemPowerState {
    pub capacity: u8,
    pub status: BatteryStatus,
    pub ac_online: bool,
}

impl Default for SystemPowerState {
    fn default() -> Self {
        let mut capacity: u8 = 100;
        let mut status: BatteryStatus = BatteryStatus::Unknown;
        let mut ac_online: bool = true;

        if let Ok(entries) = fs::read_dir("/sys/class/power_supply/") {
            for entry in entries.flatten() {
                let path = entry.path();

                if let Ok(type_str) = fs::read_to_string(path.join("type")) {
                    match type_str.trim() {
                        "Battery" => {
                            if let Ok(cap_str) = fs::read_to_string(path.join("capacity")) {
                                capacity = cap_str.trim().parse::<u8>().unwrap_or(100);
                            }
                            if let Ok(stat_str) = fs::read_to_string(path.join("status")) {
                                status = match stat_str.trim() {
                                    "Charging" => BatteryStatus::Charging,
                                    "Discharging" => BatteryStatus::Discharging,
                                    "Full" => BatteryStatus::Full,
                                    "NotCharging" => BatteryStatus::NotCharging,
                                    _ => BatteryStatus::Unknown,
                                }
                            }
                        }
                        "Mains" | "USB" | "USB_PD" | "USB_C" => {
                            if let Ok(online_str) = fs::read_to_string(path.join("online")) {
                                ac_online = online_str.trim() == "1";
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Self {
            capacity,
            status,
            ac_online,
        }
    }
}

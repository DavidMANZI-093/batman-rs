use serde::Deserialize;

#[derive(Debug)]
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

pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

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

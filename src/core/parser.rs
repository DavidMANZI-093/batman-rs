use crate::core::{BatteryStatus, ParseError, PowerEvent, models::SystemPowerState};

pub fn parse_uevent(data: &[u8]) -> Result<PowerEvent, ParseError> {
    let mut is_power_supply: bool = false;
    let mut ps_type: Option<&[u8]> = None;
    let mut capacity: Option<&[u8]> = None;
    let mut status: Option<&[u8]> = None;
    let mut online: Option<&[u8]> = None;

    for segment in data.split(|&b| b == 0) {
        if !segment.is_empty() {
            let mut parts = segment.splitn(2, |&b| b == b'=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                match (key, value) {
                    (b"SUBSYSTEM", b"power_supply") => is_power_supply = true,
                    (b"POWER_SUPPLY_TYPE", _) => ps_type = Some(value),
                    (b"POWER_SUPPLY_CAPACITY", _) => capacity = Some(value),
                    (b"POWER_SUPPLY_STATUS", _) => status = Some(value),
                    (b"POWER_SUPPLY_ONLINE", _) => online = Some(value),
                    _ => {}
                }
            }
        }
    }

    if is_power_supply && let Some(t) = ps_type {
        match t {
            b"Battery" => {
                let cap_bytes = capacity.ok_or(ParseError::MissingFields)?;
                let stat_bytes = status.ok_or(ParseError::MissingFields)?;

                let cap_str =
                    std::str::from_utf8(cap_bytes).map_err(|_| ParseError::InvalidUtf8)?;

                let cap_val = cap_str
                    .parse::<u8>()
                    .map_err(|_| ParseError::InvalidCapacity(cap_str.to_string()))?;

                let stat_str =
                    std::str::from_utf8(stat_bytes).map_err(|_| ParseError::InvalidUtf8)?;

                let stat_val = match stat_str {
                    "Charging" => BatteryStatus::Charging,
                    "Discharging" => BatteryStatus::Discharging,
                    "Full" => BatteryStatus::Full,
                    "Not charging" => BatteryStatus::NotCharging,
                    "Unknown" => BatteryStatus::Unknown,
                    _ => return Err(ParseError::InvalidStatus(stat_str.to_string())),
                };

                return Ok(PowerEvent::Battery {
                    capacity: cap_val,
                    status: stat_val,
                });
            }
            b"Mains" | b"USB" | b"USB_PD" | b"USB_C" => {
                let online_bytes = online.ok_or(ParseError::MissingFields)?;

                return Ok(PowerEvent::AcAdapter {
                    online: (online_bytes == b"1"),
                });
            }
            _ => {}
        }
    }

    Err(ParseError::MissingFields)
}

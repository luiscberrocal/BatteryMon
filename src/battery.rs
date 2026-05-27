use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct BatteryInfo {
    pub percentage: f64,
    pub state: BatteryState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryState {
    Charging,
    Discharging,
    Full,
    Unknown,
}

impl BatteryInfo {
    pub fn read() -> Option<Self> {
        // Simple direct read - avoid caching complexity
        let power_supply_dir = Path::new("/sys/class/power_supply");
        
        // Find battery directory quickly
        let dir = power_supply_dir.read_dir()
            .ok()?
            .flatten()
            .find(|entry| {
                let path = entry.path();
                fs::read_to_string(path.join("type"))
                    .map(|ty| ty.trim() == "Battery")
                    .unwrap_or(false)
            })?
            .path();
        
        // Quick reads of essential files only
        let capacity = fs::read_to_string(dir.join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse().ok())?;
        
        let state = fs::read_to_string(dir.join("status"))
            .ok()
            .map(|s| {
                let status = s.trim();
                match status {
                    "Charging" => BatteryState::Charging,
                    "Discharging" => BatteryState::Discharging,
                    "Full" => BatteryState::Full,
                    _ => BatteryState::Unknown,
                }
            })
            .unwrap_or(BatteryState::Unknown);
        
        Some(BatteryInfo { percentage: capacity, state })
    }
}

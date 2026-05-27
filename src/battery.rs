use std::fs;
use std::path::{Path, PathBuf};

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

impl BatteryState {
    pub(crate) fn parse(raw: &str) -> Self {
        match raw.trim() {
            "Charging" => BatteryState::Charging,
            "Discharging" => BatteryState::Discharging,
            "Full" => BatteryState::Full,
            _ => BatteryState::Unknown,
        }
    }
}

impl BatteryInfo {
    pub fn read() -> Option<Self> {
        Self::read_from(Path::new("/sys/class/power_supply"))
    }

    pub(crate) fn read_from(power_supply_dir: &Path) -> Option<Self> {
        let dir = Self::find_battery_dir(power_supply_dir)?;

        let capacity: f64 = fs::read_to_string(dir.join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse().ok())?;

        let state = fs::read_to_string(dir.join("status"))
            .ok()
            .map(|s| BatteryState::parse(&s))
            .unwrap_or(BatteryState::Unknown);

        Some(BatteryInfo { percentage: capacity, state })
    }

    fn find_battery_dir(power_supply_dir: &Path) -> Option<PathBuf> {
        power_supply_dir
            .read_dir()
            .ok()?
            .flatten()
            .find(|entry| {
                let path = entry.path();
                fs::read_to_string(path.join("type"))
                    .map(|ty| ty.trim() == "Battery")
                    .unwrap_or(false)
            })
            .map(|entry| entry.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "batterymon-test-{}-{}-{}",
                label,
                std::process::id(),
                nanos
            ));
            fs::create_dir_all(&path).unwrap();
            TempDir { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn make_supply(
        root: &Path,
        name: &str,
        kind: &str,
        capacity: Option<&str>,
        status: Option<&str>,
    ) {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("type"), kind).unwrap();
        if let Some(c) = capacity {
            fs::write(dir.join("capacity"), c).unwrap();
        }
        if let Some(s) = status {
            fs::write(dir.join("status"), s).unwrap();
        }
    }

    #[test]
    fn parse_state_known_values() {
        assert_eq!(BatteryState::parse("Charging"), BatteryState::Charging);
        assert_eq!(BatteryState::parse("Discharging"), BatteryState::Discharging);
        assert_eq!(BatteryState::parse("Full"), BatteryState::Full);
    }

    #[test]
    fn parse_state_trims_whitespace() {
        assert_eq!(BatteryState::parse("Charging\n"), BatteryState::Charging);
        assert_eq!(BatteryState::parse("  Full  "), BatteryState::Full);
    }

    #[test]
    fn parse_state_unknown_for_other_values() {
        assert_eq!(BatteryState::parse(""), BatteryState::Unknown);
        assert_eq!(BatteryState::parse("Not charging"), BatteryState::Unknown);
        assert_eq!(BatteryState::parse("garbage"), BatteryState::Unknown);
    }

    #[test]
    fn read_from_returns_none_when_root_missing() {
        let p = Path::new("/this/path/does/not/exist/for/batterymon/tests");
        assert!(BatteryInfo::read_from(p).is_none());
    }

    #[test]
    fn read_from_returns_none_when_no_battery_present() {
        let td = TempDir::new("no-battery");
        make_supply(&td.path, "AC", "Mains", None, None);
        assert!(BatteryInfo::read_from(&td.path).is_none());
    }

    #[test]
    fn read_from_parses_battery_entry() {
        let td = TempDir::new("with-battery");
        make_supply(&td.path, "BAT0", "Battery", Some("87\n"), Some("Discharging\n"));
        let info = BatteryInfo::read_from(&td.path).expect("should parse");
        assert_eq!(info.percentage, 87.0);
        assert_eq!(info.state, BatteryState::Discharging);
    }

    #[test]
    fn read_from_skips_non_battery_entries() {
        let td = TempDir::new("mixed");
        make_supply(&td.path, "AC", "Mains", None, None);
        make_supply(&td.path, "BAT0", "Battery", Some("42"), Some("Charging"));
        let info = BatteryInfo::read_from(&td.path).expect("should find battery");
        assert_eq!(info.percentage, 42.0);
        assert_eq!(info.state, BatteryState::Charging);
    }

    #[test]
    fn read_from_returns_none_when_capacity_missing() {
        let td = TempDir::new("no-capacity");
        make_supply(&td.path, "BAT0", "Battery", None, Some("Full"));
        assert!(BatteryInfo::read_from(&td.path).is_none());
    }

    #[test]
    fn read_from_returns_unknown_state_when_status_missing() {
        let td = TempDir::new("no-status");
        make_supply(&td.path, "BAT0", "Battery", Some("55"), None);
        let info = BatteryInfo::read_from(&td.path).expect("should still parse");
        assert_eq!(info.percentage, 55.0);
        assert_eq!(info.state, BatteryState::Unknown);
    }

    #[test]
    fn read_from_returns_none_when_capacity_unparseable() {
        let td = TempDir::new("bad-capacity");
        make_supply(&td.path, "BAT0", "Battery", Some("notanumber"), Some("Full"));
        assert!(BatteryInfo::read_from(&td.path).is_none());
    }
}

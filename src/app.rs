use cosmic::app::Core;
use cosmic::iced::{
    platform_specific::shell::commands::popup::{destroy_popup, get_popup},
    window::Id, Limits,
};
use cosmic::iced_runtime::core::window;
use cosmic::{Action, Element, Task};
use cosmic::widget::{text, column, settings, slider};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::battery::{BatteryInfo, BatteryState};

const ID: &str = "io.github.BatteryMon";

/// Width the panel label is padded to so the applet button doesn't reflow
/// as the text alternates between e.g. `Batt:100%` and `Batt:Critical`.
pub(crate) const LABEL_WIDTH: usize = 12;

pub(crate) const DEFAULT_HIBERNATE_THRESHOLD: f32 = 20.0;
pub(crate) const DEFAULT_SHUTDOWN_THRESHOLD: f32 = 10.0;
pub(crate) const ANIMATION_SWITCH_INTERVAL: Duration = Duration::from_secs(5);
pub(crate) const POPUP_OPEN_TICK_DIVISOR: u32 = 5;

#[derive(Default)]
pub struct App {
    core: Core,
    popup: Option<Id>,
    battery_info: Arc<RwLock<Option<BatteryInfo>>>,
    hibernate_threshold: f32,
    shutdown_threshold: f32,
    show_text: bool,
    last_text_switch_time: Option<Instant>,
    tick_counter: u32,
    // Temporary settings (only apply when window closes)
    temp_hibernate_threshold: Option<f32>,
    temp_shutdown_threshold: Option<f32>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Tick,
    TogglePopup,
    PopupClosed(Id),
    UpdateLowThreshold(f32),
    UpdateCriticalThreshold(f32),
    RestoreDefaults,
}

/// Build the panel label for the given battery snapshot.
///
/// When discharging below the critical threshold, alternates between the
/// percentage and `Batt:Critical` (controlled by `show_text`); below the
/// hibernate threshold, alternates with `Batt:Low`. Charging or unknown
/// states always show the percentage. Missing info renders `Batt:N/A`.
/// Output is always padded to [`LABEL_WIDTH`].
pub(crate) fn format_battery_label(
    info: Option<&BatteryInfo>,
    hibernate_threshold: f32,
    shutdown_threshold: f32,
    show_text: bool,
) -> String {
    let Some(info) = info else {
        return format!("{:<width$}", "Batt:N/A", width = LABEL_WIDTH);
    };

    let percent_text = format!("Batt:{:.0}%", info.percentage);

    let label = if info.state == BatteryState::Discharging {
        if info.percentage <= shutdown_threshold as f64 {
            if show_text { percent_text } else { "Batt:Critical".to_string() }
        } else if info.percentage <= hibernate_threshold as f64 {
            if show_text { percent_text } else { "Batt:Low".to_string() }
        } else {
            percent_text
        }
    } else {
        percent_text
    };

    format!("{:<width$}", label, width = LABEL_WIDTH)
}

/// True when the label should animate between percentage and Low/Critical.
pub(crate) fn is_animation_active(
    info: Option<&BatteryInfo>,
    hibernate_threshold: f32,
    shutdown_threshold: f32,
) -> bool {
    let Some(info) = info else { return false; };
    info.state == BatteryState::Discharging
        && (info.percentage <= shutdown_threshold as f64
            || info.percentage <= hibernate_threshold as f64)
}

/// True if enough time has passed since the last animation flip
/// (or no flip has happened yet) that we should toggle `show_text`.
pub(crate) fn should_switch_text(last_switch: Option<Instant>, now: Instant) -> bool {
    match last_switch {
        None => true,
        Some(t) => now.duration_since(t) >= ANIMATION_SWITCH_INTERVAL,
    }
}

/// Returns `(new_tick_counter, should_update_battery)`.
///
/// When the popup is open we only sample the battery every fifth tick to
/// avoid menu hangs while sliders are being dragged; when it's closed we
/// sample every tick for real-time updates.
pub(crate) fn tick_should_update_battery(counter: u32, popup_open: bool) -> (u32, bool) {
    if popup_open {
        let next = (counter + 1) % POPUP_OPEN_TICK_DIVISOR;
        (next, next == 0)
    } else {
        (0, true)
    }
}

impl cosmic::Application for App {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        let battery_info = Arc::new(RwLock::new(BatteryInfo::read()));

        let app = App {
            core,
            popup: None,
            battery_info,
            hibernate_threshold: DEFAULT_HIBERNATE_THRESHOLD,
            shutdown_threshold: DEFAULT_SHUTDOWN_THRESHOLD,
            show_text: true,
            last_text_switch_time: None,
            tick_counter: 0,
            temp_hibernate_threshold: None,
            temp_shutdown_threshold: None,
        };

        let task = Task::perform(async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Message::Tick
        }, |action| Action::App(action));

        (app, task)
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<Action<Self::Message>> {
        match message {
            Message::Tick => {
                let (next_counter, should_update_battery) =
                    tick_should_update_battery(self.tick_counter, self.popup.is_some());
                self.tick_counter = next_counter;

                if should_update_battery {
                    if let Some(battery_info) = crate::battery::BatteryInfo::read() {
                        if let Ok(mut info) = self.battery_info.write() {
                            *info = Some(battery_info);
                        }
                    }
                }

                let snapshot = self
                    .battery_info
                    .read()
                    .ok()
                    .and_then(|guard| (*guard).clone());

                if is_animation_active(
                    snapshot.as_ref(),
                    self.hibernate_threshold,
                    self.shutdown_threshold,
                ) {
                    let now = Instant::now();
                    if should_switch_text(self.last_text_switch_time, now) {
                        self.show_text = !self.show_text;
                        self.last_text_switch_time = Some(now);
                    }
                } else {
                    self.show_text = true;
                    self.last_text_switch_time = None;
                }

                return Task::perform(tokio::time::sleep(Duration::from_secs(1)), |_| Message::Tick)
                    .then(|msg| Task::perform(async { msg }, |action| Action::App(action)));
            }
            Message::TogglePopup => {
                if let Some(popup_id) = self.popup.take() {
                    return destroy_popup(popup_id);
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(800.0)
                        .min_width(700.0)
                        .min_height(400.0)
                        .max_height(800.0);
                    return get_popup(popup_settings);
                }
            }
            Message::PopupClosed(popup_id) => {
                if self.popup.as_ref() == Some(&popup_id) {
                    if let Some(temp_low) = self.temp_hibernate_threshold {
                        self.hibernate_threshold = temp_low;
                    }
                    if let Some(temp_critical) = self.temp_shutdown_threshold {
                        self.shutdown_threshold = temp_critical;
                    }
                    self.temp_hibernate_threshold = None;
                    self.temp_shutdown_threshold = None;
                    self.popup = None;
                }
            }
            Message::UpdateLowThreshold(value) => {
                self.temp_hibernate_threshold = Some(value);
            }
            Message::UpdateCriticalThreshold(value) => {
                self.temp_shutdown_threshold = Some(value);
            }
            Message::RestoreDefaults => {
                self.temp_hibernate_threshold = Some(DEFAULT_HIBERNATE_THRESHOLD);
                self.temp_shutdown_threshold = Some(DEFAULT_SHUTDOWN_THRESHOLD);
            }
        }

        Task::perform(async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Message::Tick
        }, |action| Action::App(action))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let info = self
            .battery_info
            .read()
            .ok()
            .and_then(|guard| (*guard).clone());

        let battery_text = format_battery_label(
            info.as_ref(),
            self.hibernate_threshold,
            self.shutdown_threshold,
            self.show_text,
        );

        let text_content = text(battery_text).size(14);
        let button_content = self.core.applet.text_button(text_content, Message::TogglePopup);
        self.core.applet.autosize_window(button_content).into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let info = self
            .battery_info
            .read()
            .ok()
            .and_then(|guard| (*guard).clone());

        let content = if let Some(ref info) = info {
            column()
                .push(
                    settings::item(
                        "Battery Level",
                        text(format!("{:.0}%", info.percentage)).size(16)
                    )
                )
                .push(
                    settings::item(
                        "Status",
                        text(format!("{:?}", info.state)).size(14)
                    )
                )
                .push(
                    settings::section()
                        .title("Notification Settings")
                        .add(
                            settings::item(
                                format!("Low Battery At: {:.0}%", self.temp_hibernate_threshold.unwrap_or(self.hibernate_threshold)),
                                slider(5.0..=95.0, self.temp_hibernate_threshold.unwrap_or(self.hibernate_threshold), Message::UpdateLowThreshold)
                                    .step(5.0)
                            )
                        )
                        .add(
                            settings::item(
                                format!("Critical Battery At: {:.0}%", self.temp_shutdown_threshold.unwrap_or(self.shutdown_threshold)),
                                slider(5.0..=95.0, self.temp_shutdown_threshold.unwrap_or(self.shutdown_threshold), Message::UpdateCriticalThreshold)
                                    .step(5.0)
                            )
                        )
                        .add(
                            settings::item(
                                "Restore Default Settings",
                                self.core.applet.text_button("◉", Message::RestoreDefaults)
                            )
                        )
                )
                .spacing(8)
                .padding(16)
        } else {
            column()
                .push(
                    settings::item(
                        "Battery Status",
                        text("Information unavailable").size(14)
                    )
                )
                .padding(16)
        };

        self.core.applet.popup_container(content).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(percentage: f64, state: BatteryState) -> BatteryInfo {
        BatteryInfo { percentage, state }
    }

    #[test]
    fn label_missing_info_shows_na_padded() {
        let label = format_battery_label(None, 20.0, 10.0, true);
        assert_eq!(label, format!("{:<12}", "Batt:N/A"));
        assert_eq!(label.len(), LABEL_WIDTH);
    }

    #[test]
    fn label_charging_always_shows_percent() {
        let b = info(8.0, BatteryState::Charging);
        // Even at 8% (below both thresholds), charging never animates.
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, false),
            format!("{:<12}", "Batt:8%")
        );
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, true),
            format!("{:<12}", "Batt:8%")
        );
    }

    #[test]
    fn label_full_state_shows_percent() {
        let b = info(100.0, BatteryState::Full);
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, false),
            format!("{:<12}", "Batt:100%")
        );
    }

    #[test]
    fn label_discharging_above_thresholds_shows_percent() {
        let b = info(50.0, BatteryState::Discharging);
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, true),
            format!("{:<12}", "Batt:50%")
        );
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, false),
            format!("{:<12}", "Batt:50%")
        );
    }

    #[test]
    fn label_discharging_at_or_below_low_alternates() {
        let b = info(15.0, BatteryState::Discharging);
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, true),
            format!("{:<12}", "Batt:15%")
        );
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, false),
            format!("{:<12}", "Batt:Low")
        );
    }

    #[test]
    fn label_discharging_at_exactly_low_threshold_is_low() {
        let b = info(20.0, BatteryState::Discharging);
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, false),
            format!("{:<12}", "Batt:Low")
        );
    }

    #[test]
    fn label_discharging_at_or_below_critical_alternates_with_critical() {
        let b = info(5.0, BatteryState::Discharging);
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, true),
            format!("{:<12}", "Batt:5%")
        );
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, false),
            format!("{:<12}", "Batt:Critical")
        );
    }

    #[test]
    fn label_critical_takes_precedence_over_low() {
        // Below both thresholds — must say Critical, never Low.
        let b = info(5.0, BatteryState::Discharging);
        assert_eq!(
            format_battery_label(Some(&b), 20.0, 10.0, false),
            format!("{:<12}", "Batt:Critical")
        );
    }

    #[test]
    fn label_padding_is_always_label_width() {
        let cases = [
            format_battery_label(None, 20.0, 10.0, true),
            format_battery_label(Some(&info(100.0, BatteryState::Full)), 20.0, 10.0, true),
            format_battery_label(Some(&info(5.0, BatteryState::Discharging)), 20.0, 10.0, false),
            format_battery_label(Some(&info(15.0, BatteryState::Discharging)), 20.0, 10.0, false),
        ];
        for label in cases {
            assert!(
                label.len() >= LABEL_WIDTH,
                "label {label:?} should be padded to at least {LABEL_WIDTH} chars"
            );
        }
    }

    #[test]
    fn animation_inactive_when_info_missing() {
        assert!(!is_animation_active(None, 20.0, 10.0));
    }

    #[test]
    fn animation_inactive_when_charging_even_if_low() {
        let b = info(5.0, BatteryState::Charging);
        assert!(!is_animation_active(Some(&b), 20.0, 10.0));
    }

    #[test]
    fn animation_inactive_when_full() {
        let b = info(100.0, BatteryState::Full);
        assert!(!is_animation_active(Some(&b), 20.0, 10.0));
    }

    #[test]
    fn animation_inactive_when_discharging_above_threshold() {
        let b = info(50.0, BatteryState::Discharging);
        assert!(!is_animation_active(Some(&b), 20.0, 10.0));
    }

    #[test]
    fn animation_active_when_discharging_at_or_below_low() {
        let b = info(20.0, BatteryState::Discharging);
        assert!(is_animation_active(Some(&b), 20.0, 10.0));
        let b = info(15.0, BatteryState::Discharging);
        assert!(is_animation_active(Some(&b), 20.0, 10.0));
    }

    #[test]
    fn animation_active_when_discharging_below_critical() {
        let b = info(5.0, BatteryState::Discharging);
        assert!(is_animation_active(Some(&b), 20.0, 10.0));
    }

    #[test]
    fn should_switch_text_when_never_switched() {
        assert!(should_switch_text(None, Instant::now()));
    }

    #[test]
    fn should_not_switch_text_within_interval() {
        let now = Instant::now();
        assert!(!should_switch_text(Some(now), now));
        let recent = now - Duration::from_secs(2);
        assert!(!should_switch_text(Some(recent), now));
    }

    #[test]
    fn should_switch_text_after_interval() {
        let now = Instant::now();
        let old = now - ANIMATION_SWITCH_INTERVAL;
        assert!(should_switch_text(Some(old), now));
        let older = now - Duration::from_secs(10);
        assert!(should_switch_text(Some(older), now));
    }

    #[test]
    fn tick_when_popup_closed_always_updates_and_resets_counter() {
        for starting in [0, 1, 2, 3, 4, 99] {
            let (next, update) = tick_should_update_battery(starting, false);
            assert_eq!(next, 0);
            assert!(update);
        }
    }

    #[test]
    fn tick_when_popup_open_updates_every_fifth_tick() {
        // Starting from 0 (just opened), the next four ticks should NOT
        // update, and the fifth should.
        let mut counter = 0u32;
        let mut updates = Vec::new();
        for _ in 0..10 {
            let (next, update) = tick_should_update_battery(counter, true);
            counter = next;
            updates.push(update);
        }
        assert_eq!(
            updates,
            vec![false, false, false, false, true, false, false, false, false, true]
        );
    }
}

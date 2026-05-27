use cosmic::app::Core;
use cosmic::iced::{
    platform_specific::shell::commands::popup::{destroy_popup, get_popup},
    window::Id, Limits,
};
use cosmic::iced_runtime::core::window;
use cosmic::{Action, Element, Task};
use cosmic::widget::{text, column, settings, slider};
use std::sync::{Arc, RwLock};

use crate::battery::{BatteryInfo, BatteryState};

const ID: &str = "io.github.BatteryMon";

#[derive(Default)]
pub struct App {
    core: Core,
    popup: Option<Id>,
    battery_info: Arc<RwLock<Option<BatteryInfo>>>,
    hibernate_threshold: f32,
    shutdown_threshold: f32,
    show_text: bool,
    last_text_switch_time: Option<std::time::Instant>, // Track last text switch time
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
            hibernate_threshold: 20.0,
            shutdown_threshold: 10.0,
            show_text: true,
            last_text_switch_time: None,
            // Temporary settings (only apply when window closes)
            temp_hibernate_threshold: None,
            temp_shutdown_threshold: None,
        };

        let task = Task::perform(async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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
                // Update battery less frequently when menu is open to prevent slowdown
                let should_update_battery = if self.popup.is_some() {
                    // Only update every 5 ticks (5 seconds) when menu is open
                    static mut TICK_COUNTER: u32 = 0;
                    unsafe {
                        TICK_COUNTER += 1;
                        if TICK_COUNTER >= 5 {
                            TICK_COUNTER = 0;
                            true
                        } else {
                            false
                        }
                    }
                } else {
                    true // Update every tick when menu is closed
                };
                
                if should_update_battery {
                    if let Some(battery_info) = crate::battery::BatteryInfo::read() {
                        if let Ok(mut info) = self.battery_info.write() {
                            *info = Some(battery_info);
                        }
                    }
                }
                
                // Simple time-based text animation (always work for notifications)
                let should_animate = if let Ok(info) = self.battery_info.read() {
                    if let Some(ref info) = *info {
                        info.state == BatteryState::Discharging && (
                            info.percentage <= self.shutdown_threshold as f64 || 
                            info.percentage <= self.hibernate_threshold as f64
                        )
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                if should_animate {
                    let now = std::time::Instant::now();
                    let should_switch = if let Some(last_switch) = self.last_text_switch_time {
                        now.duration_since(last_switch) >= std::time::Duration::from_secs(5)
                    } else {
                        true
                    };
                    
                    if should_switch {
                        self.show_text = !self.show_text;
                        self.last_text_switch_time = Some(now);
                    }
                } else {
                    self.show_text = true;
                    self.last_text_switch_time = None;
                }
                
                // Use consistent 1-second tick rate
                return Task::perform(tokio::time::sleep(std::time::Duration::from_secs(1)), |_| Message::Tick)
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
                        .max_width(800.0)  // Much wider window for sliders (was 600)
                        .min_width(700.0)  // Minimum width (was 500)
                        .min_height(400.0)  // Taller window (was 350)
                        .max_height(800.0);
                    return get_popup(popup_settings);
                }
            }
            Message::PopupClosed(popup_id) => {
                if self.popup.as_ref() == Some(&popup_id) {
                    // Apply temporary settings when window closes
                    if let Some(temp_low) = self.temp_hibernate_threshold {
                        self.hibernate_threshold = temp_low;
                    }
                    if let Some(temp_critical) = self.temp_shutdown_threshold {
                        self.shutdown_threshold = temp_critical;
                    }
                    
                    // Clear temporary values
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
                self.temp_hibernate_threshold = Some(20.0);
                self.temp_shutdown_threshold = Some(10.0);
            }
        }
        
        // Always continue the timer
        Task::perform(async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            Message::Tick
        }, |action| Action::App(action))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let info = self.battery_info.read()
            .ok()
            .and_then(|guard| (*guard).clone());
        
        let battery_text = if let Some(ref info) = info {
            if info.state == BatteryState::Discharging {
                if info.percentage <= self.shutdown_threshold as f64 {
                    // Critical battery - alternate between percentage and "Critical"
                    if self.show_text {
                        let text = format!("Batt:{:.0}%", info.percentage);
                        // Pad to consistent width (max: "Batt:100%")
                        format!("{:<12}", text)
                    } else {
                        // Pad to consistent width
                        format!("{:<12}", "Batt:Critical")
                    }
                } else if info.percentage <= self.hibernate_threshold as f64 {
                    // Low battery - alternate between percentage and "Low"
                    if self.show_text {
                        let text = format!("Batt:{:.0}%", info.percentage);
                        // Pad to consistent width
                        format!("{:<12}", text)
                    } else {
                        // Pad to consistent width
                        format!("{:<12}", "Batt:Low")
                    }
                } else {
                    // Normal battery - show percentage
                    let text = format!("Batt:{:.0}%", info.percentage);
                    // Pad to consistent width
                    format!("{:<12}", text)
                }
            } else {
                // Charging or other states - show percentage
                let text = format!("Batt:{:.0}%", info.percentage);
                // Pad to consistent width
                format!("{:<12}", text)
            }
        } else {
            // Pad to consistent width
            format!("{:<12}", "Batt:N/A")
        };
        
        // Simple text content - working version
        let text_content = text(battery_text.clone()).size(14);
        
        let button_content = self.core.applet.text_button(text_content, Message::TogglePopup);
        
        self.core.applet.autosize_window(button_content).into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        // Cache battery info to avoid repeated reads when menu is open
        let info = self.battery_info.read()
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

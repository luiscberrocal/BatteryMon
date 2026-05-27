# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

BatteryMon is a battery monitoring **panel applet for the COSMIC desktop** (Pop!_OS), written in Rust on top of `libcosmic` (pinned to a specific `pop-os/libcosmic` git rev in `Cargo.toml`). The repo ships both source and a pre-compiled binary so it can be installed without a Rust toolchain.

## Common commands

```bash
# Build the optimized release binary (overwrites ./batterymon when run from project root only if cargo target is configured; default target is target/release/batterymon)
cargo build --release

# Quick type/dependency check without a full build
cargo check

# Install to ~/.local/bin. Picks target/release/batterymon if present
# (i.e. you just ran `cargo build --release`), otherwise falls back to
# the bundled ./batterymon at the repo root.
./install.sh

# Uninstall
./uninstall.sh

# Restart the COSMIC panel after install/uninstall to load the applet
cosmic-panel &
```

There are no tests or lint configs in this repo. `cargo fmt` / `cargo clippy` work via the standard Rust toolchain but no project-specific config exists.

`install.sh` prefers `$PROJECT_DIR/target/release/batterymon` if it exists and otherwise falls back to the bundled `$PROJECT_DIR/batterymon`. A fresh `cargo build --release` will therefore be picked up automatically — no need to copy it over the root binary.

## Architecture

Three source files in `src/`:

- `main.rs` — entry point; just calls `cosmic::applet::run::<App>(())`.
- `app.rs` — the `App` struct implementing `cosmic::Application`. Holds applet state, the popup window id, thresholds, and an `Arc<RwLock<Option<BatteryInfo>>>` cache of the latest battery reading. Defines `Message` enum (`Tick`, `TogglePopup`, `PopupClosed`, `UpdateLowThreshold`, `UpdateCriticalThreshold`, `RestoreDefaults`) and the iced `update` / `view` / `view_window` handlers.
- `battery.rs` — pure sysfs reader. Scans `/sys/class/power_supply/*` for the first entry whose `type` file is `Battery`, then reads `capacity` and `status`. No external crate; no caching at this layer.

### Tick loop and adaptive update rate

The app drives itself with a self-rescheduling `Message::Tick` (sleep 1s → re-emit Tick). The tick handler does two things:

1. **Battery refresh** — every tick when the popup is closed; only every 5th tick when the popup is open. This is implemented with an `unsafe static mut TICK_COUNTER` counter inside `update()`. Be aware: this is a deliberate optimization to avoid menu hangs while sliders are being dragged. Touching this code without preserving the open/closed cadence will regress that.
2. **Low/critical animation** — when battery is `Discharging` and below either threshold, the label alternates between `Batt:NN%` and `Batt:Low` / `Batt:Critical` every 5 seconds. Animation is driven by `last_text_switch_time` so it works regardless of menu state.

### Settings: temporary vs applied

Slider edits write to `temp_hibernate_threshold` / `temp_shutdown_threshold` (`Option<f32>`). The real fields (`hibernate_threshold`, `shutdown_threshold`) are only updated in `Message::PopupClosed`. This intentional two-stage commit avoids re-rendering the whole popup on every slider tick (which was the v1.1 "slider freezing" bug). Preserve this pattern when adding new settings.

Defaults: `hibernate_threshold = 20.0`, `shutdown_threshold = 10.0`. Slider range 5–95 step 5. `RestoreDefaults` sets the temp values, so it only takes effect when the popup closes.

### Display width

The label is padded to 12 chars (`format!("{:<12}", ...)`) so the applet button doesn't reflow as the text changes between `Batt:100%`, `Batt:Low`, etc. Keep new label states within this width.

## libcosmic dependency

`libcosmic` is pinned to git rev `9270358` of `pop-os/libcosmic` with the `applet` feature. The API surface is unstable — if you bump the rev, expect to update imports under `cosmic::iced::platform_specific::shell::commands::popup`, the `Application` trait, and the `applet::*` helpers.

## Release profile

`Cargo.toml` configures `strip`, `lto`, `codegen-units = 1`, `panic = "abort"`, `opt-level = "z"`. These are tuned for binary size (~11MB stripped). Don't loosen them casually — the README advertises the small binary as a feature.

## Files to ignore when reading the code

- `src/app.rs.backup` — an older v1.4-ish snapshot of `app.rs` kept for reference; not built and out of sync with current `app.rs`.

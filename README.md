# BatteryMon — COSMIC Battery Monitor Applet

A small battery monitoring applet for the **COSMIC desktop** (Pop!_OS), written
in Rust on top of [`libcosmic`](https://github.com/pop-os/libcosmic). Shows the
current battery percentage in the panel and animates a `Low` / `Critical` label
when the level drops below user-configurable thresholds.

> **Credits**: This project is a fork / refinement of the original
> [DkYSwe/BatteryMon](https://github.com/DkYSwe/BatteryMon/tree/main).
> Refer to the upstream repository for the original implementation and history.

---

## Requirements

### Runtime
- **Pop!_OS / COSMIC desktop** (the applet is loaded by `cosmic-panel`).
- A Linux kernel that exposes batteries under `/sys/class/power_supply/*`
  (every mainstream laptop kernel does — no extra service needed).
- `~/.local/bin` on your `PATH` (the installer writes here; it does **not**
  need root).

### Build-from-source
You only need these if you intend to rebuild the binary. If you just want to
run the applet, skip ahead to [Install](#install).

- **Rust toolchain** — stable, 1.75 or newer is recommended. Install via
  [`rustup`](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup update stable
  ```
- **System build dependencies** (libcosmic links against several system
  libraries). On Pop!_OS / Ubuntu:
  ```bash
  sudo apt install build-essential pkg-config libssl-dev \
                   libxkbcommon-dev libwayland-dev libfontconfig1-dev
  ```
- **Git** — `cargo` fetches `libcosmic` directly from GitHub.
- ~2 GB of free disk for the build artifacts in `target/`.

---

## Install

There are two supported flows. Pick **one**.

### Option A — Build from source (recommended)

```bash
# 1. Clone
git clone https://github.com/<your-fork>/BatteryMon.git
cd BatteryMon

# 2. Build the optimized release binary
#    Produces ./target/release/batterymon (~11 MB stripped)
cargo build --release

# 3. Install to ~/.local/bin and copy the desktop file + icons
./install.sh

# 4. Restart the COSMIC panel so it picks up the new applet
cosmic-panel &
```

`install.sh` reads the binary from `./target/release/batterymon` (the output of
step 2). If that file is missing it will fail with a clear error.

### Option B — Use the pre-built binary checked into the repo

The repo ships a pre-compiled `./batterymon` so you can install without a Rust
toolchain. To use it, copy it into the path `install.sh` expects:

```bash
mkdir -p target/release
cp batterymon target/release/batterymon
./install.sh
cosmic-panel &
```

> Note: the bundled binary is built against a specific glibc on Pop!_OS. If it
> fails to start (e.g. `GLIBC_X.YY not found`), fall back to **Option A**.

### What `install.sh` does

1. Copies the binary to `~/.local/bin/batterymon` **and**
   `~/.local/bin/cosmic-applet-batterymon` (the second name is what
   `cosmic-panel` looks for).
2. Copies `data/io.github.BatteryMon.desktop` to
   `~/.local/share/applications/`.
3. Copies icons from `data/icons/` into `~/.local/share/icons/`.
4. Kills any running `cosmic-panel` so you can restart it cleanly.

No `sudo` is required, and nothing is written outside your home directory.

### Adding the applet to the panel

After `cosmic-panel &`:

1. Right-click the COSMIC panel → **Configure Panel** (or **Add Applet**).
2. Find **BatteryMon** in the list and add it.

---

## Uninstall

```bash
./uninstall.sh
cosmic-panel &
```

This removes the binaries from `~/.local/bin`, the desktop file, and the
installed icons.

---

## Usage

- Click the applet to open the popup.
- Use the sliders to set the **Low** and **Critical** thresholds
  (5–95 %, step 5). Changes are committed when the popup closes.
- Click **◉** to restore defaults (Low = 20 %, Critical = 10 %).
- When the battery is **discharging** and below a threshold, the label
  alternates between `Batt:NN%` and `Batt:Low` / `Batt:Critical` every 5 s.

The label is fixed-width (12 chars) so the panel does not reflow as the text
changes.

---

## Project layout

```
src/
  main.rs       Entry point — hands off to cosmic::applet::run
  app.rs        Applet state, popup, sliders, tick loop, animation
  battery.rs    Pure sysfs reader (/sys/class/power_supply)
data/
  io.github.BatteryMon.desktop
  icons/        hicolor icon tree
install.sh / uninstall.sh
Cargo.toml      libcosmic pinned to rev 9270358
```

See `CLAUDE.md` for deeper architecture notes (tick cadence, two-stage
slider commit, release profile rationale).

---

## Troubleshooting

**Applet doesn't appear after restart**
```bash
which cosmic-applet-batterymon       # should print ~/.local/bin/...
pgrep cosmic-applet-batterymon       # should return a PID once added
```
If `which` prints nothing, add `~/.local/bin` to your `PATH`:
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Build fails on `libcosmic` / `cosmic-text`**
The `Cargo.toml` patches `cosmic-text` from `vendor/cosmic-text` to work around
upstream API drift. Make sure you cloned the repo with that vendored directory
intact (it's checked in, not a submodule).

**Binary works but battery always shows 0 %**
Check that `/sys/class/power_supply/` contains an entry whose `type` file
reads `Battery`. Desktops and some VMs have no battery and the applet will
show `N/A`.

---

## License

Dual-licensed under MIT or Apache-2.0, matching the upstream
[DkYSwe/BatteryMon](https://github.com/DkYSwe/BatteryMon/tree/main) project.

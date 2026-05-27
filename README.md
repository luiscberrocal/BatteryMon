# BatteryMon - COSMIC Battery Monitor Applet

## Complete Release Package (Final - Bulletproof Installation)

This package contains both the **pre-compiled binary** for immediate installation AND the **complete source code** for building from scratch.

## Latest Version: v1.5 - Bulletproof Installation

### ✅ **Installation Script Fixed:**
- **Smart file detection** - Finds desktop file in multiple locations
- **Graceful fallbacks** - Continues installation even if files are missing
- **Detailed logging** - Shows what's found and what's installed
- **Error handling** - Provides helpful debugging information
- **Icon installation** - Robust icon copying with fallbacks

## Quick Installation (Pre-compiled)

### Easy Install
```bash
# Extract the archive
unzip BatteryMon-complete-release.zip
cd BatteryMon-complete-release

# Install to user directory (bulletproof script)
chmod +x install.sh
./install.sh

# Restart COSMIC panel manually
cosmic-panel &
```

### What the Install Script Does:
1. **Finds all required files** in multiple possible locations
2. **Installs binary** to `~/.local/bin/` (both names)
3. **Installs desktop file** if found (in `data/` or root)
4. **Installs icons** if found (in `data/icons/` or `icons/`)
5. **Shows detailed progress** with ✓ and ⚠ indicators
6. **Provides troubleshooting** tips if anything fails

## Build from Source

### Prerequisites
- Rust and Cargo (latest stable)
- COSMIC desktop environment
- Basic build tools

### Build Instructions
```bash
# Extract the archive
unzip BatteryMon-complete-release.zip
cd BatteryMon-complete-release

# Build the applet (optimized)
cargo build --release

# Install the built binary (bulletproof script)
chmod +x install.sh
./install.sh

# Restart COSMIC panel manually
cosmic-panel &
```

## Features
- **Real-time battery monitoring** with percentage display
- **Visual alerts** for low and critical battery levels  
- **Customizable thresholds** (5-95% for low and critical)
- **Time-based text animation** when battery is low/critical
- **Fixed-width display** - No layout shifts
- **Restore defaults** - Easy reset to original settings
- **Optimized performance** - No hanging when power disconnected
- **Smart updates** - Menu updates every 5 seconds when open, 1 second when closed
- **Clean, minimal interface** without unnecessary complexity
- **User-directory installation** (no root access required)
- **Small binary size** - Optimized for distribution (11MB)
- **Working notifications** - Low/critical alerts work in all states
- **Bulletproof installation** - Smart file detection and graceful fallbacks

## Usage
1. Right-click the BatteryMon applet in your COSMIC panel
2. Adjust low and critical battery thresholds using the sliders
3. Settings are applied when you close the settings window
4. Click "◉" to restore defaults to 20%/10%
5. The applet will show battery status and animate when thresholds are reached

## Notification Behavior
- **Low Battery (≤20%)**: Alternates between "Batt:85%" and "Batt:Low" every 5 seconds
- **Critical Battery (≤10%)**: Alternates between "Batt:8%" and "Batt:Critical" every 5 seconds
- **Works with menu open/closed** - Notifications always visible
- **Charging state** - Shows normal percentage, no animation

## Menu Performance
- **Menu Open:** Updates every 5 seconds (prevents hanging)
- **Menu Closed:** Updates every 1 second (real-time monitoring)
- **Power Changes:** Status updates within 5 seconds in menu
- **No Hanging:** Smooth operation even when unplugging cable
- **Notifications Active:** Animation works regardless of menu state

## Installation Script Features

### ✅ Smart File Detection:
```bash
# Desktop file locations checked:
- $PROJECT_DIR/data/io.github.BatteryMon.desktop
- $PROJECT_DIR/io.github.BatteryMon.desktop
- $PROJECT_DIR/data/io.github.BatteryMon.desktop

# Icon directories checked:
- $PROJECT_DIR/data/icons
- $PROJECT_DIR/icons
```

### ✅ Graceful Fallbacks:
- **Missing desktop file** - Continues without desktop integration
- **Missing icons** - Continues without icon installation
- **Missing binary** - Shows error with directory listing
- **Permission issues** - Provides helpful troubleshooting

### ✅ Detailed Output:
```
BatteryMon Installation Script
==============================
Project directory: /path/to/BatteryMon-complete-release
✓ Binary found: /path/to/BatteryMon-complete-release/batterymon
✓ Desktop file found: /path/to/BatteryMon-complete-release/data/io.github.BatteryMon.desktop
✓ Icons directory found: /path/to/BatteryMon-complete-release/data/icons

Installing binary to /home/user/.local/bin
✓ Binary installed successfully
Installing desktop file to /home/user/.local/share/applications
✓ Desktop file installed successfully
Installing icons to /home/user/.local/share/icons
✓ Icons installed successfully
```

## Files Included

### Pre-compiled Binary (11MB - Optimized)
- `batterymon` - Ready-to-use binary (stripped & optimized)

### Source Code
- `src/` - Complete source code
  - `main.rs` - Application entry point
  - `app.rs` - Main application logic and UI (v1.4 with notifications fix)
  - `battery.rs` - Battery information reading (streamlined)
- `Cargo.toml` - Rust project configuration (with optimizations)

### Installation Files
- `install.sh` - **Bulletproof installation script** (v1.5)
- `uninstall.sh` - Uninstallation script
- `data/` - Desktop file and icons
  - `io.github.BatteryMon.desktop` - Desktop file
  - `icons/` - Icon files for all sizes

## Troubleshooting

### Installation Issues?
The bulletproof install script provides detailed feedback:

#### **If Desktop File Not Found:**
```
Warning: Desktop file not found in expected locations
Expected locations:
  /path/to/data/io.github.BatteryMon.desktop
  /path/to/io.github.BatteryMon.desktop
  /path/to/data/io.github.BatteryMon.desktop

Continuing without desktop file installation...
```

#### **If Icons Not Found:**
```
Warning: Icons directory not found in expected locations
Expected locations:
  /path/to/data/icons
  /path/to/icons

Continuing without icon installation...
```

#### **Installation Summary:**
```
Installation summary:
- Binary: /home/user/.local/bin/batterymon ✓
- Desktop file: /home/user/.local/share/applications/io.github.BatteryMon.desktop ✓
- Icons: /home/user/.local/share/icons ✓
```

### Applet Invisible After Installation?
```bash
# Check if binary is in PATH
which cosmic-applet-batterymon

# Check if process is running
pgrep cosmic-applet-batterymon

# Add to PATH if needed
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Restart panel
cosmic-panel &
```

### Notifications Not Working?
This version (v1.4) fixes the notification issue:
- ✅ Low/critical alerts work regardless of menu state
- ✅ Animation restored for battery warnings
- ✅ Performance optimizations maintained

### Build Issues?
```bash
# Update Rust
rustup update stable

# Clean build
cargo clean
cargo build --release

# Check dependencies
cargo check
```

## Uninstall
```bash
cd BatteryMon-complete-release
chmod +x uninstall.sh
./uninstall.sh

# Restart COSMIC panel manually
cosmic-panel &
```

## Requirements
- COSMIC desktop environment
- User permissions for ~/.local/bin and ~/.local/share/applications
- Rust and Cargo (for building from source)

## Technical Details

### Performance Optimizations
- **Adaptive tick rate** - 1s when menu closed, 5s when menu open
- **Conditional battery reads** - Reduced file system access
- **Smart animation** - Works in all menu states
- **Error handling** - Graceful power state transitions

### Binary Optimizations
- **Debug symbols stripped** - 60% size reduction (28MB → 11MB)
- **Link-time optimization** - Better code generation
- **Size optimization** - Focused on minimal binary size
- **Minimal dependencies** - Only required tokio features

### Installation Script Robustness
- **Multi-location file detection** - Finds files in different directory structures
- **Graceful degradation** - Continues installation even with missing components
- **Detailed logging** - Clear feedback on what's found and installed
- **Error recovery** - Helpful troubleshooting information
- **Cross-platform compatibility** - Works with different system configurations

### v1.5 Changes
- **Bulletproof install script** - Smart file detection and fallbacks
- **Enhanced error handling** - Better debugging information
- **Graceful installation** - Works even if some files are missing
- **Detailed progress reporting** - Clear success/failure indicators

## Version History
- **v1.5** - Bulletproof installation script (final release)
- **v1.4** - Fixed low/critical notifications
- **v1.3** - Complete release with source + binary, 60% size reduction
- **v1.2** - Fixed menu hanging, adaptive updates
- **v1.1** - Fixed slider freezing, COSMIC UI improvements
- **v1.0** - Initial release

## Support
This package includes everything needed to use or modify BatteryMon:
- ✅ Ready-to-use binary for immediate installation (11MB optimized)
- ✅ Complete source code for customization
- ✅ All dependencies and build files
- ✅ Bulletproof installation script with smart file detection
- ✅ Installation and uninstallation scripts
- ✅ Icons and desktop integration
- ✅ Working notifications in all states
- ✅ Detailed troubleshooting information

Enjoy your fully functional battery monitoring experience with bulletproof installation! 🎉

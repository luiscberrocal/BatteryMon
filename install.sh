#!/bin/bash
# Installation script for BatteryMon applet

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Prefer a freshly-built binary at target/release/batterymon; otherwise fall
# back to the pre-built ./batterymon checked into the repo. This lets users
# install either by building from source (`cargo build --release`) or by using
# the bundled binary, without any extra copy step.
BUILT_BINARY="$PROJECT_DIR/target/release/batterymon"
BUNDLED_BINARY="$PROJECT_DIR/batterymon"
if [ -f "$BUILT_BINARY" ]; then
    BINARY_PATH="$BUILT_BINARY"
    BINARY_SOURCE="freshly built (target/release/batterymon)"
elif [ -f "$BUNDLED_BINARY" ]; then
    BINARY_PATH="$BUNDLED_BINARY"
    BINARY_SOURCE="bundled pre-built (./batterymon)"
else
    BINARY_PATH="$BUILT_BINARY"  # used only for the error message below
    BINARY_SOURCE=""
fi

# Look for desktop file in multiple possible locations
DESKTOP_FILE_LOCATIONS=(
    "$PROJECT_DIR/data/io.github.BatteryMon.desktop"
    "$PROJECT_DIR/io.github.BatteryMon.desktop"
)

# Find the desktop file
DESKTOP_FILE=""
for location in "${DESKTOP_FILE_LOCATIONS[@]}"; do
    if [ -f "$location" ]; then
        DESKTOP_FILE="$location"
        break
    fi
done

# Look for icons directory
ICON_DIR_LOCATIONS=(
    "$PROJECT_DIR/data/icons"
    "$PROJECT_DIR/icons"
)

# Find the icons directory
ICON_DIR=""
for location in "${ICON_DIR_LOCATIONS[@]}"; do
    if [ -d "$location" ]; then
        ICON_DIR="$location"
        break
    fi
done

echo "BatteryMon Installation Script"
echo "=============================="
echo "Project directory: $PROJECT_DIR"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: No batterymon binary found."
    echo "Looked in:"
    echo "  $BUILT_BINARY    (built from source)"
    echo "  $BUNDLED_BINARY  (pre-built, shipped with the repo)"
    echo ""
    echo "Either build it with:"
    echo "  cargo build --release"
    echo "or restore the bundled ./batterymon from the repo."
    exit 1
fi

echo "✓ Binary found: $BINARY_PATH ($BINARY_SOURCE)"

# Check if desktop file exists
if [ -z "$DESKTOP_FILE" ]; then
    echo "Warning: Desktop file not found in expected locations"
    echo "Expected locations:"
    printf '  %s\n' "${DESKTOP_FILE_LOCATIONS[@]}"
    echo ""
    echo "Current directory structure:"
    find "$PROJECT_DIR" -name "*.desktop" -type f 2>/dev/null || echo "No .desktop files found"
    echo ""
    echo "Continuing without desktop file installation..."
else
    echo "✓ Desktop file found: $DESKTOP_FILE"
fi

# Check if icons directory exists
if [ -z "$ICON_DIR" ]; then
    echo "Warning: Icons directory not found in expected locations"
    echo "Expected locations:"
    printf '  %s\n' "${ICON_DIR_LOCATIONS[@]}"
    echo ""
    echo "Continuing without icon installation..."
else
    echo "✓ Icons directory found: $ICON_DIR"
fi

echo ""

# Install binary to ~/.local/bin (user directory only)
INSTALL_BIN="$HOME/.local/bin"
mkdir -p "$INSTALL_BIN"
echo "Installing binary to $INSTALL_BIN"
cp "$BINARY_PATH" "$INSTALL_BIN/batterymon"
chmod +x "$INSTALL_BIN/batterymon"

# Create symlink for COSMIC applet naming
cp "$BINARY_PATH" "$INSTALL_BIN/cosmic-applet-batterymon"
chmod +x "$INSTALL_BIN/cosmic-applet-batterymon"
echo "✓ Binary installed successfully"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$INSTALL_BIN:"* ]]; then
    echo ""
    echo "Note: $INSTALL_BIN is not in your PATH."
    echo "Add this to your ~/.bashrc or ~/.profile:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

# Install desktop file if found
if [ -n "$DESKTOP_FILE" ]; then
    DESKTOP_DIR="$HOME/.local/share/applications"
    mkdir -p "$DESKTOP_DIR"
    echo "Installing desktop file to $DESKTOP_DIR"
    cp "$DESKTOP_FILE" "$DESKTOP_DIR/io.github.BatteryMon.desktop"
    echo "✓ Desktop file installed successfully"

    # Update desktop database
    if command -v update-desktop-database &> /dev/null; then
        echo "Updating desktop database..."
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    fi
fi

# Install icons if found
if [ -n "$ICON_DIR" ]; then
    ICON_INSTALL_DIR="$HOME/.local/share/icons"
    mkdir -p "$ICON_INSTALL_DIR"
    echo "Installing icons to $ICON_INSTALL_DIR"
    
    # Copy all icon files recursively
    if command -v rsync &> /dev/null; then
        rsync -av "$ICON_DIR/" "$ICON_INSTALL_DIR/" 2>/dev/null || true
    else
        cp -r "$ICON_DIR"/* "$ICON_INSTALL_DIR/" 2>/dev/null || true
    fi
    echo "✓ Icons installed successfully"
fi

# Kill COSMIC panel (user will need to restart manually)
echo ""
echo "Killing COSMIC panel to prevent duplicate panels..."
killall cosmic-panel 2>/dev/null || true
echo "✓ COSMIC panel killed"

echo ""
echo "Installation complete!"
echo "====================="
echo ""
echo "BatteryMon has been installed to your user directory."
echo "COSMIC panel has been killed to prevent duplicates."
echo ""
echo "Please restart COSMIC panel manually:"
echo "  cosmic-panel &"
echo ""
echo "After restarting, if you don't see BatteryMon in your panel:"
echo "1. Right-click on the COSMIC panel"
echo "2. Select 'Add Applet' or 'Configure Panel'"
echo "3. Find 'BatteryMon' in the list"
echo "4. Click to add it to your panel"
echo ""
echo "Installation summary:"
echo "- Binary: $INSTALL_BIN/batterymon ✓"
if [ -n "$DESKTOP_FILE" ]; then
    echo "- Desktop file: $HOME/.local/share/applications/io.github.BatteryMon.desktop ✓"
else
    echo "- Desktop file: SKIPPED (not found) ⚠"
fi
if [ -n "$ICON_DIR" ]; then
    echo "- Icons: $ICON_INSTALL_DIR ✓"
else
    echo "- Icons: SKIPPED (not found) ⚠"
fi
echo ""
echo "Alternative troubleshooting:"
echo "- Log out and log back in"
echo "- Ensure ~/.local/bin is in your PATH"
echo "- Check if cosmic-applet-batterymon is running: pgrep cosmic-applet-batterymon"

#!/bin/bash
# Uninstallation script for BatteryMon applet

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Uninstalling BatteryMon..."

# Stop running applet
echo "Stopping BatteryMon applet..."
killall cosmic-applet-batterymon 2>/dev/null || true
killall batterymon 2>/dev/null || true

# Remove binary from user directory only
if [ -f "$HOME/.local/bin/batterymon" ]; then
    echo "Removing binary from ~/.local/bin"
    rm -f "$HOME/.local/bin/batterymon"
fi

if [ -f "$HOME/.local/bin/cosmic-applet-batterymon" ]; then
    echo "Removing applet binary from ~/.local/bin"
    rm -f "$HOME/.local/bin/cosmic-applet-batterymon"
fi

# Remove desktop file
if [ -f "$HOME/.local/share/applications/io.github.BatteryMon.desktop" ]; then
    echo "Removing desktop file"
    rm -f "$HOME/.local/share/applications/io.github.BatteryMon.desktop"
fi

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    echo "Updating desktop database..."
    update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
fi

# Kill COSMIC panel (user will need to restart manually)
echo "Killing COSMIC panel to prevent duplicate panels..."
killall cosmic-panel 2>/dev/null || true
echo "COSMIC panel killed. Please restart it manually when ready:"
echo "  cosmic-panel &"

echo ""
echo "Uninstallation complete!"
echo ""
echo "BatteryMon has been completely removed from your system."
echo "COSMIC panel has been killed to prevent duplicates."
echo ""
echo "Please restart COSMIC panel manually:"
echo "  cosmic-panel &"
echo ""
echo "If you still see BatteryMon in your panel after restarting:"
echo "1. Right-click on the applet"
echo "2. Select 'Remove' or 'Delete'"
echo "3. Or log out and log back in"

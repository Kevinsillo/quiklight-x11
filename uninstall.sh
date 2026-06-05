#!/usr/bin/env bash
# Quiklight (Rust) — uninstaller
set -euo pipefail

BIN_DST="$HOME/.local/bin/quiklight"
DATA_DIR="$HOME/.local/share/quiklight-rust"
CONFIG_DST="$DATA_DIR/config.toml"
SERVICE="$HOME/.config/systemd/user/quiklight.service"
RULES="/etc/udev/rules.d/99-quiklight.rules"

echo "==> Uninstalling Quiklight (Rust)"

# Stop and remove the service (ExecStop blanks the strip)
systemctl --user disable --now quiklight 2>/dev/null || true
rm -f "$SERVICE"
systemctl --user daemon-reload

# Remove the udev rule
sudo rm -f "$RULES"
sudo udevadm control --reload-rules

# Remove the installed binary
rm -f "$BIN_DST"

# Leave the data dir alone: on this machine it doubles as the source repo.
# Only the config is project-owned, and we keep it so settings survive a reinstall.
if [ -f "$CONFIG_DST" ]; then
    echo "    Kept config at $CONFIG_DST (delete it by hand if you want a clean slate)"
fi

echo "✓ Quiklight removed (binary, service and udev rule)."

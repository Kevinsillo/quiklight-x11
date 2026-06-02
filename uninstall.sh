#!/usr/bin/env bash
# Quiklight X11 — uninstaller
set -euo pipefail

INSTALL_DIR="$HOME/.local/share/quiklight"
SERVICE="$HOME/.config/systemd/user/quiklight.service"
RULES="/etc/udev/rules.d/99-quiklight.rules"

echo "==> Uninstalling Quiklight X11"

systemctl --user disable --now quiklight 2>/dev/null || true
rm -f "$SERVICE"
systemctl --user daemon-reload

sudo rm -f "$RULES"
sudo udevadm control --reload-rules

rm -rf "$INSTALL_DIR"

echo "✓ Quiklight removed."

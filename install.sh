#!/usr/bin/env bash
# Quiklight X11 — installer
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="$HOME/.local/share/quiklight"
SERVICE_DIR="$HOME/.config/systemd/user"
RULES_DST="/etc/udev/rules.d/99-quiklight.rules"
VENV="$INSTALL_DIR/.venv"

echo "==> Installing Quiklight X11"

# ── 1. Copy project files ───────────────────────────────────────────────────
mkdir -p "$INSTALL_DIR"
cp -r "$REPO/quiklight"        "$INSTALL_DIR/"
cp    "$REPO/requirements.txt" "$INSTALL_DIR/"
echo "    Files copied to $INSTALL_DIR"

# ── 2. Python virtual environment ──────────────────────────────────────────
echo "==> Creating virtual environment"
python3 -m venv "$VENV"
"$VENV/bin/pip" install --quiet --upgrade pip
"$VENV/bin/pip" install --quiet -r "$INSTALL_DIR/requirements.txt"
echo "    Dependencies installed"

# ── 3. udev rule (allows access to the HID device without root) ─────────────
echo "==> Installing udev rule (requires sudo)"
sudo cp "$REPO/99-quiklight.rules" "$RULES_DST"
sudo udevadm control --reload-rules
sudo udevadm trigger
echo "    udev rule installed at $RULES_DST"

# ── 4. systemd user service ─────────────────────────────────────────────────
echo "==> Installing systemd service"
mkdir -p "$SERVICE_DIR"
cat > "$SERVICE_DIR/quiklight.service" <<EOF
[Unit]
Description=Quiklight X11 ambilight driver
After=graphical-session.target

[Service]
ExecStart=$VENV/bin/python -m quiklight
WorkingDirectory=$INSTALL_DIR
Restart=on-failure
RestartSec=3
RestartPreventExitStatus=0
Environment=DISPLAY=:0

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now quiklight
echo "    Service enabled and started"

echo ""
echo "✓ Quiklight is running. Check status with:"
echo "    systemctl --user status quiklight"
echo ""
echo "  Edit settings in: $INSTALL_DIR/quiklight/config.py"
echo "  Then restart:     systemctl --user restart quiklight"

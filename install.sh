#!/usr/bin/env bash
# Quiklight (Rust) — installer
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
BIN_DST="$BIN_DIR/quiklight"
DATA_DIR="$HOME/.local/share/quiklight-rust"
CONFIG_DST="$DATA_DIR/config.toml"
SERVICE_DIR="$HOME/.config/systemd/user"
RULES_DST="/etc/udev/rules.d/99-quiklight.rules"

echo "==> Installing Quiklight (Rust)"

# ── 1. Build the release binary ─────────────────────────────────────────────
echo "==> Building release binary (cargo build --release)"
cargo build --release --manifest-path "$REPO/Cargo.toml"

# ── 2. Install the binary to a runnable path ────────────────────────────────
mkdir -p "$BIN_DIR"
install -m 0755 "$REPO/target/release/quiklight" "$BIN_DST"
echo "    Binary installed at $BIN_DST"

# ── 3. Default config (never overwrite an existing one) ─────────────────────
mkdir -p "$DATA_DIR"
if [ -f "$CONFIG_DST" ]; then
    echo "    Keeping existing config at $CONFIG_DST"
elif [ "$REPO/config.toml" -ef "$CONFIG_DST" ]; then
    echo "    Config already in place at $CONFIG_DST"
else
    cp "$REPO/config.toml" "$CONFIG_DST"
    echo "    Default config written to $CONFIG_DST"
fi

# ── 4. udev rule (lets the HID device be opened without root) ────────────────
echo "==> Installing udev rule (requires sudo)"
sudo cp "$REPO/99-quiklight.rules" "$RULES_DST"
sudo udevadm control --reload-rules
sudo udevadm trigger
echo "    udev rule installed at $RULES_DST"

# ── 5. systemd user service ─────────────────────────────────────────────────
echo "==> Installing systemd service"
mkdir -p "$SERVICE_DIR"
cat > "$SERVICE_DIR/quiklight.service" <<EOF
[Unit]
Description=Quiklight ambilight driver (Rust)
After=graphical-session.target

[Service]
ExecStart=$BIN_DST
ExecStop=$BIN_DST --off
Restart=on-failure
RestartSec=3
RestartPreventExitStatus=0
Environment=DISPLAY=:0
Environment=XAUTHORITY=%h/.Xauthority

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
echo "  List monitors:   quiklight --list-monitors"
echo "  Change options:  quiklight --monitor DP-4 --fps 30 --save"
echo "  Edit config at:  $CONFIG_DST"
echo "  Apply changes:   systemctl --user restart quiklight"
case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo ""
       echo "  Note: $BIN_DIR is not on your PATH. Add this to your shell profile:"
       echo "        export PATH=\"\$HOME/.local/bin:\$PATH\"" ;;
esac

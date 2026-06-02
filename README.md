# Quiklight X11

An ambilight driver for the **Robobloq Quiklight** USB LED strip on Linux X11 desktops.

Captures your screen edges in real time and syncs the colors to the LED strip — with no cloud, no Electron, no browser stack.

> **Wayland user?** Check out [hypr-quicklight](https://github.com/shim80/hypr-quicklight), which this project is based on for protocol research.

---

## Compatible hardware

Any device that identifies as:

| Field | Value |
|---|---|
| Vendor ID | `0x1A86` (QinHeng Electronics) |
| Product ID | `0xFE07` |
| Manufacturer string | `ROBOBLOQ` |
| Product string | `USBHID` |

Sold as **Robobloq Quiklight** and variants. The default layout assumes **63 LEDs** (17 left + 29 top + 17 right). If your strip has a different layout, adjust `config.py`.

---

## Requirements

- Linux with **X11** (Xorg) — XFCE, GNOME, KDE, etc.
- Python **3.10+**
- `sudo` access (for the udev rule — one time only)
- The Quiklight USB device plugged in

---

## Installation

```bash
git clone https://github.com/kevinsillo/quiklight-x11.git
cd quiklight-x11
chmod +x install.sh
./install.sh
```

The installer:
1. Copies project files to `~/.local/share/quiklight/`
2. Creates a Python virtual environment and installs dependencies
3. Installs a udev rule so the device is accessible without root
4. Registers and starts a systemd user service that runs at login

---

## Configuration

Edit `~/.local/share/quiklight/quiklight/config.py`:

```python
FPS        = 60     # frames per second
BRIGHTNESS = 200    # device brightness (0–255)
SMOOTHING  = 0.15   # temporal smoothing (0.0 = off, 0.95 = very smooth)
SATURATION = 1.4    # color vividity (1.0 = natural, 2.0 = very vivid)
GAMMA      = 1.2    # gamma correction (1.0 = linear)

# LED strip layout
LEFT  = 17          # LEDs on the left side
TOP   = 29          # LEDs across the top
RIGHT = 17          # LEDs on the right side
```

Apply changes:
```bash
systemctl --user restart quiklight
```

---

## Service management

```bash
systemctl --user status quiklight     # check status and recent logs
systemctl --user restart quiklight    # apply config changes
systemctl --user stop quiklight       # turn off LEDs and stop
systemctl --user disable quiklight    # prevent autostart at login
```

---

## Troubleshooting

**Device not found**
```
RuntimeError: Quiklight device not found (VID=0x1A86, PID=0xFE07)
```
- Make sure the USB cable is plugged in
- Re-run the installer or manually install the udev rule:
  ```bash
  sudo cp 99-quiklight.rules /etc/udev/rules.d/
  sudo udevadm control --reload-rules && sudo udevadm trigger
  ```

**Permission denied on `/dev/hidrawN`**
- The udev rule is not active. Unplug and replug the device, then try again.

**LEDs are out of order or sides are inverted**
- Adjust `LEFT`, `TOP`, `RIGHT` counts in `config.py` to match your physical strip.
- If sides are mirrored, the LED order in `capture.py` can be reversed per segment.

**High CPU usage**
- Lower `FPS` (e.g. `30`) or increase `CAP_W`/`CAP_H` capture resolution minimally.

---

## Uninstall

```bash
cd quiklight-x11
./uninstall.sh
```

---

## How it works

1. Captures the primary monitor at a very low resolution (80×45 px by default) using `mss`
2. Samples the border pixels for each LED zone
3. Applies saturation boost and gamma correction
4. Blends with the previous frame (smoothing)
5. Encodes the colors using the device's proprietary `RB`/`SC` HID protocol
6. Writes 65-byte HID reports to `/dev/hidrawN`

The protocol was reverse-engineered from [hypr-quicklight](https://github.com/jgferrer/hypr-quicklight).

---

## License

MIT

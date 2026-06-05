# Quiklight (Rust)

An ambilight driver for the **Robobloq Quiklight** USB LED strip on Linux X11 desktops.

Captures your screen edges in real time and syncs the colors to the LED strip — with no cloud, no Electron, no browser stack. This is a Rust rewrite of [quiklight-x11](https://github.com/kevinsillo/quiklight-x11) (Python), aimed at lower latency and lower CPU usage.

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

Sold as **Robobloq Quiklight** and variants. The default layout assumes **63 LEDs** (17 left + 29 top + 17 right). If your strip has a different layout, adjust `config.toml` or pass `--top/--right/--left`.

---

## Requirements

- Linux with **X11** (Xorg) — XFCE, GNOME, KDE, etc.
- **Rust toolchain** (`cargo`) — install from <https://rustup.rs>
- `sudo` access (for the udev rule — one time only)
- The Quiklight USB device plugged in

---

## Installation

```bash
git clone https://github.com/kevinsillo/quiklight-rust.git
cd quiklight-rust
chmod +x install.sh
./install.sh
```

The installer:
1. Builds the release binary (`cargo build --release`)
2. Installs it to `~/.local/bin/quiklight`
3. Writes a default `config.toml` to `~/.local/share/quiklight-rust/` (existing config is kept)
4. Installs a udev rule so the device is accessible without root
5. Registers and starts a systemd user service that runs at login

---

## Configuration

Two ways to configure: the **config file** (persistent defaults) or **CLI flags** (per run, or persisted with `--save`).

### Config file

Edit `~/.local/share/quiklight-rust/config.toml`:

```toml
# LED strip layout (Robobloq Quiklight default: 63 LEDs)
top   = 29
right = 17
left  = 17

# Screen capture resolution — lower = faster, less precise
cap_w  = 80
cap_h  = 45
border = 3

# Which monitor to capture (omit for the primary monitor)
# monitor = "DP-4"

# Performance
fps = 60

# Color processing
brightness = 200   # global brightness sent to device (0–255)
smoothing  = 0.15  # temporal smoothing (0.0 = off, 0.95 = very smooth)
saturation = 1.4   # color saturation boost (1.0 = none, 2.0 = vivid)
gamma      = 1.2   # gamma correction (1.0 = linear)
```

Apply changes:
```bash
systemctl --user restart quiklight
```

### CLI

The binary ships a full CLI (`quiklight --help`). Flags override the config for that run; add `--save` to persist them.

```
Usage: quiklight [OPTIONS]

Options:
  -m, --monitor <INDEX|NAME>  Capture this monitor instead of the primary one
  -f, --fps <N>               Target frames per second (1-240)
  -r, --resolution <WxH>      Capture resolution (see --list-resolutions)
  -b, --brightness <0-255>    Global brightness sent to the device
  -s, --smoothing <F>         Temporal smoothing: 0.0 = off … 0.95 = very smooth
      --saturation <F>        Saturation boost: 1.0 = none, 2.0 = vivid
      --gamma <F>             Gamma correction: 1.0 = linear, >1 darkens midtones
      --border <PX>           Border thickness sampled at each edge
      --top/--right/--left N  LEDs per edge
  -c, --config <FILE>         Use a different config file
      --list-monitors         List detected monitors and exit
      --list-resolutions      List capture-resolution presets and exit
      --save                  Persist the resulting options to the config file
      --off                   Turn the LEDs off (send a black frame) and exit
  -h, --help                  Print help
  -V, --version               Print version
```

Examples:

```bash
quiklight --list-monitors                 # see available screens
quiklight --list-resolutions -m DP-4      # resolution presets for a screen
quiklight -m DP-4 -f 30 -r 96x54          # try options without saving
quiklight -m DP-4 -r 80x45 --save         # persist to config.toml and run
quiklight --off                           # blank the strip
```

**Multi-monitor:** by default the **primary** monitor is captured (detected via RandR). Use `--monitor` with an index or name from `--list-monitors` to pick another.

**Resolution presets:** `--list-resolutions` shows integer-divisor resolutions of the monitor's native size (same aspect ratio, always clean multiples), from native down to tiny. Lower resolution = less CPU.

---

## Service management

```bash
systemctl --user status quiklight     # check status and recent logs
systemctl --user restart quiklight    # apply config changes
systemctl --user stop quiklight       # turn off LEDs and stop (ExecStop sends a black frame)
systemctl --user disable quiklight    # prevent autostart at login
```

---

## Troubleshooting

**Device not found**
```
Quiklight device not found (VID=0x1A86, PID=0xFE07).
```
- Make sure the USB cable is plugged in
- Re-run the installer or manually install the udev rule:
  ```bash
  sudo cp 99-quiklight.rules /etc/udev/rules.d/
  sudo udevadm control --reload-rules && sudo udevadm trigger
  ```

**Permission denied on `/dev/hidrawN`**
- The udev rule is not active. Unplug and replug the device, then try again.

**One side stays a fixed color on a multi-monitor setup**
- The wrong screen is being captured. Run `quiklight --list-monitors`, then select the
  right one: `quiklight --monitor DP-4 --save && systemctl --user restart quiklight`.

**LEDs are out of order or sides are inverted**
- Adjust `top`, `right`, `left` to match your physical strip (config file or flags).

**High CPU usage**
- Lower `fps` (e.g. `30`) or pick a smaller `--resolution` preset.

---

## Uninstall

```bash
cd quiklight-rust
./uninstall.sh
```

Removes the binary, the systemd service and the udev rule. Your `config.toml` is kept.

---

## How it works

1. Detects the **primary monitor** via RandR (or the one given with `--monitor`)
2. Captures it at a very low resolution (80×45 px by default) using **X11 XShm**
3. Samples the border pixels for each LED zone (left → top → right)
4. Applies saturation boost and gamma correction
5. Blends with the previous frame (smoothing)
6. Encodes the colors using the device's proprietary `RB`/`SC` HID protocol
7. Writes 65-byte HID reports directly to `/dev/hidrawN` (no libhidapi)

The protocol was reverse-engineered from [hypr-quicklight](https://github.com/shim80/hypr-quicklight).

---

## License

MIT

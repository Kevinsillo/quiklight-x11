# Quiklight X11 — tunable parameters
# Edit this file to customize behaviour. Restart the service after changes:
#   systemctl --user restart quiklight

# --- LED strip layout (Robobloq Quiklight default: 63 LEDs) ---
TOP   = 29   # LEDs across the top
RIGHT = 17   # LEDs on the right side
LEFT  = 17   # LEDs on the left side
LEDS  = TOP + RIGHT + LEFT

# --- Screen capture ---
CAP_W  = 80  # capture width  (pixels) — lower = faster, less precise
CAP_H  = 45  # capture height (pixels)
BORDER = 3   # border thickness to sample (pixels in the reduced image)

# --- Performance ---
FPS = 60     # target frames per second

# --- Color processing ---
BRIGHTNESS = 200   # global brightness sent to device  (0–255)
SMOOTHING  = 0.15  # temporal smoothing  (0.0 = off, 0.95 = very smooth)
SATURATION = 1.4   # color saturation boost  (1.0 = none, 2.0 = vivid)
GAMMA      = 1.2   # gamma correction  (1.0 = linear, >1 darkens midtones)

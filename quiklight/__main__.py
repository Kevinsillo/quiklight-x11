#!/usr/bin/env python3
# Quiklight X11 — entry point  (run with: python -m quiklight)
import time
import warnings

warnings.filterwarnings("ignore")

from mss import MSS

from . import capture, config, protocol
from .device import find_hidraw


def main() -> None:
    hidraw = find_hidraw()
    fd = open(hidraw, "rb+", buffering=0)
    msg_id = protocol.init(fd)
    interval = 1.0 / config.FPS
    prev = [(0, 0, 0)] * config.LEDS

    print(
        f"Quiklight running on {hidraw} — "
        f"{config.LEDS} LEDs @ {config.FPS} fps | "
        f"smoothing={config.SMOOTHING}  saturation={config.SATURATION}  gamma={config.GAMMA}"
    )

    with MSS() as sct:
        monitor = sct.monitors[1]
        while True:
            t0 = time.monotonic()

            leds = capture.grab(sct, monitor)
            leds = capture.smooth(leds, prev, config.SMOOTHING)
            prev = leds

            protocol.send_frame(fd, msg_id & 0xFF, leds)
            msg_id += 1

            wait = interval - (time.monotonic() - t0)
            if wait > 0:
                time.sleep(wait)


if __name__ == "__main__":
    main()

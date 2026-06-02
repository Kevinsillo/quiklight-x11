# Quiklight X11 — screen capture and color processing
import numpy as np

from . import config


def grab(sct, monitor) -> list[tuple[int, int, int]]:
    """Capture the screen borders and return one RGB tuple per LED."""
    raw = np.array(sct.grab(monitor))[:, :, :3]  # BGR
    h_full, w_full = raw.shape[:2]
    img = raw[::h_full // config.CAP_H, ::w_full // config.CAP_W, :][:config.CAP_H, :config.CAP_W, :]

    leds: list[tuple[int, int, int]] = []

    # Left side (top → bottom) — LEDs 1-17
    col = img[:, :config.BORDER, :]
    for i in range(config.LEFT):
        y0 = int(i * config.CAP_H / config.LEFT)
        y1 = max(y0 + 1, int((i + 1) * config.CAP_H / config.LEFT))
        b, g, r = col[y0:y1].mean(axis=(0, 1))
        leds.append(_process(int(r), int(g), int(b)))

    # Top (left → right) — LEDs 18-46
    row = img[:config.BORDER, :, :]
    for i in range(config.TOP):
        x0 = int(i * config.CAP_W / config.TOP)
        x1 = max(x0 + 1, int((i + 1) * config.CAP_W / config.TOP))
        b, g, r = row[:, x0:x1].mean(axis=(0, 1))
        leds.append(_process(int(r), int(g), int(b)))

    # Right side (top → bottom) — LEDs 47-63
    col = img[:, -config.BORDER:, :]
    for i in range(config.RIGHT):
        y0 = int(i * config.CAP_H / config.RIGHT)
        y1 = max(y0 + 1, int((i + 1) * config.CAP_H / config.RIGHT))
        b, g, r = col[y0:y1].mean(axis=(0, 1))
        leds.append(_process(int(r), int(g), int(b)))

    return leds


def smooth(
    current: list[tuple[int, int, int]],
    prev: list[tuple[int, int, int]],
    factor: float,
) -> list[tuple[int, int, int]]:
    """Blend current frame with the previous one to reduce flickering."""
    return [
        (
            int(factor * pr + (1 - factor) * r),
            int(factor * pg + (1 - factor) * g),
            int(factor * pb + (1 - factor) * b),
        )
        for (r, g, b), (pr, pg, pb) in zip(current, prev)
    ]


def _process(r: int, g: int, b: int) -> tuple[int, int, int]:
    rf, gf, bf = r / 255.0, g / 255.0, b / 255.0

    # Saturation boost
    mx = max(rf, gf, bf)
    mn = min(rf, gf, bf)
    if mx > 0:
        sat = (mx - mn) / mx
        new_sat = min(1.0, sat * config.SATURATION)
        if sat > 0:
            scale = new_sat / sat
            rf = max(0.0, mx - (mx - rf) * scale)
            gf = max(0.0, mx - (mx - gf) * scale)
            bf = max(0.0, mx - (mx - bf) * scale)

    # Gamma correction
    inv_gamma = 1.0 / config.GAMMA
    return (
        int(min(255, rf ** inv_gamma * 255)),
        int(min(255, gf ** inv_gamma * 255)),
        int(min(255, bf ** inv_gamma * 255)),
    )

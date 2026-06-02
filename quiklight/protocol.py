# Quiklight X11 — proprietary RB/SC HID protocol
from . import config


def _checksum(data: list[int]) -> int:
    return sum(data) & 0xFF


def _send(fd, packet: bytes) -> None:
    offset = 0
    while offset < len(packet):
        chunk = packet[offset:offset + 64]
        fd.write(b"\x00" + chunk + bytes(64 - len(chunk)))
        offset += 64


def _rb(msg_id: int, action: int, payload: list[int]) -> bytes:
    total = 2 + 1 + 1 + 1 + len(payload) + 1
    out = [ord("R"), ord("B"), total, msg_id, action] + payload
    out.append(_checksum(out))
    return bytes(out)


def _frame(msg_id: int, leds: list[tuple[int, int, int]]) -> bytes:
    payload_len = config.LEDS * 5
    total = 2 + 2 + 1 + 1 + payload_len + 1
    out = [ord("S"), ord("C"), (total >> 8) & 0xFF, total & 0xFF, msg_id, 128]
    for i, (r, g, b) in enumerate(leds):
        n = i + 1
        out += [n, r, g, b, n]
    out.append(_checksum(out))
    return bytes(out)


def init(fd) -> int:
    """Send initialization sequence. Returns the next message ID."""
    mid = 0
    _send(fd, _rb(mid, 147, [0]));                                      mid += 1  # setOpenUrl
    _send(fd, _rb(mid, 135, [config.BRIGHTNESS]));                      mid += 1  # setBrightness
    _send(fd, _rb(mid, 134, [1, 85, 85, 85, 63, 64, 0, 0, 0, 254]));  mid += 1  # setSectionLED
    _send(fd, _frame(mid, [(0, 0, 0)] * config.LEDS));                  mid += 1  # black frame
    return mid


def send_frame(fd, msg_id: int, leds: list[tuple[int, int, int]]) -> None:
    _send(fd, _frame(msg_id, leds))

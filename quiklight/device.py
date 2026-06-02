# Quiklight X11 — HID device auto-detection
import glob
import os

VENDOR_ID  = 0x1A86
PRODUCT_ID = 0xFE07


def find_hidraw(vid: int = VENDOR_ID, pid: int = PRODUCT_ID) -> str:
    """Return the first /dev/hidrawN matching the given VID/PID.

    Raises RuntimeError if the device is not found.
    """
    target = f"0003:{vid:08X}:{pid:08X}".upper()
    candidates: list[str] = []

    for dev_path in glob.glob("/sys/bus/hid/devices/*"):
        uevent = os.path.join(dev_path, "uevent")
        try:
            content = open(uevent).read().upper()
        except OSError:
            continue
        if target in content:
            for hidraw in glob.glob(os.path.join(dev_path, "hidraw", "hidraw*")):
                candidates.append(f"/dev/{os.path.basename(hidraw)}")

    if not candidates:
        raise RuntimeError(
            f"Quiklight device not found (VID=0x{vid:04X}, PID=0x{pid:04X}).\n"
            "Check that the device is plugged in and the udev rule is installed."
        )

    return sorted(candidates)[0]  # lowest index → interface 0 (LED control)

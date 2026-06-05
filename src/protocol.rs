use anyhow::Result;
use std::io::Write;

fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

fn write_hid(out: &mut impl Write, packet: &[u8]) -> Result<()> {
    let mut offset = 0;
    while offset < packet.len() {
        let end = (offset + 64).min(packet.len());
        let chunk = &packet[offset..end];
        let mut buf = [0u8; 65];
        buf[1..1 + chunk.len()].copy_from_slice(chunk); // buf[0] = 0x00 (report ID)
        out.write_all(&buf)?;
        offset += 64;
    }
    Ok(())
}

fn rb(msg_id: u8, action: u8, payload: &[u8]) -> Vec<u8> {
    let total = 5 + payload.len() + 1;
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&[b'R', b'B', total as u8, msg_id, action]);
    out.extend_from_slice(payload);
    let cs = checksum(&out);
    out.push(cs);
    out
}

fn frame(msg_id: u8, leds: &[(u8, u8, u8)]) -> Vec<u8> {
    let payload_len = leds.len() * 5;
    let total = 6 + payload_len + 1;
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&[
        b'S',
        b'C',
        ((total >> 8) & 0xFF) as u8,
        (total & 0xFF) as u8,
        msg_id,
        128,
    ]);
    for (i, &(r, g, b)) in leds.iter().enumerate() {
        let n = (i + 1) as u8;
        out.extend_from_slice(&[n, r, g, b, n]);
    }
    let cs = checksum(&out);
    out.push(cs);
    out
}

pub fn init(out: &mut impl Write, brightness: u8, n_leds: usize) -> Result<u8> {
    let mut mid: u8 = 0;
    write_hid(out, &rb(mid, 147, &[0]))?;
    mid = mid.wrapping_add(1);
    write_hid(out, &rb(mid, 135, &[brightness]))?;
    mid = mid.wrapping_add(1);
    write_hid(out, &rb(mid, 134, &[1, 85, 85, 85, 63, 64, 0, 0, 0, 254]))?;
    mid = mid.wrapping_add(1);
    write_hid(out, &frame(mid, &vec![(0, 0, 0); n_leds]))?;
    mid = mid.wrapping_add(1);
    Ok(mid)
}

pub fn send_frame(out: &mut impl Write, msg_id: u8, leds: &[(u8, u8, u8)]) -> Result<()> {
    write_hid(out, &frame(msg_id, leds))
}

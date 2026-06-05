use anyhow::{bail, Result};
use libc::{IPC_CREAT, IPC_PRIVATE, IPC_RMID};
use std::ptr;
use x11rb::connection::Connection;
use x11rb::protocol::randr::ConnectionExt as RandrExt;
use x11rb::protocol::shm::ConnectionExt as ShmExt;
use x11rb::protocol::xproto::ConnectionExt as XprotoExt;
use x11rb::protocol::xproto::ImageFormat;
use x11rb::rust_connection::RustConnection;

use crate::config::Config;

pub struct Capturer {
    conn: RustConnection,
    root: u32,
    mx: i16,    // monitor x offset
    my: i16,    // monitor y offset
    mw: usize,  // monitor width  (pixels)
    mh: usize,  // monitor height (pixels)
    bpp: usize, // bytes per pixel
    shm_addr: *mut u8,
    shm_seg: u32,
}

/// One physical monitor as reported by RandR.
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub x: i16,
    pub y: i16,
    pub width: usize,
    pub height: usize,
    pub primary: bool,
}

/// Which monitor to capture.
#[derive(Debug, Clone)]
pub enum MonitorSelect {
    Primary,
    Index(usize),
    Name(String),
}

fn fetch_monitors(conn: &RustConnection, root: u32) -> Result<Vec<MonitorInfo>> {
    let reply = conn.randr_get_monitors(root, true)?.reply()?;
    let mut out = Vec::with_capacity(reply.monitors.len());
    for (i, m) in reply.monitors.iter().enumerate() {
        let name = conn.get_atom_name(m.name)?.reply()?;
        out.push(MonitorInfo {
            index: i,
            name: String::from_utf8_lossy(&name.name).into_owned(),
            x: m.x,
            y: m.y,
            width: m.width as usize,
            height: m.height as usize,
            primary: m.primary,
        });
    }
    Ok(out)
}

/// List every active monitor (opens its own short-lived X11 connection).
pub fn list_monitors() -> Result<Vec<MonitorInfo>> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    let root = conn.setup().roots[screen_num].root;
    fetch_monitors(&conn, root)
}

/// Pick a monitor from a list according to the selection (primary / index / name).
pub fn pick_monitor<'a>(
    mons: &'a [MonitorInfo],
    select: &MonitorSelect,
) -> Result<&'a MonitorInfo> {
    match select {
        MonitorSelect::Primary => mons
            .iter()
            .find(|m| m.primary)
            .or_else(|| mons.first())
            .ok_or_else(|| anyhow::anyhow!("No monitors found")),
        MonitorSelect::Index(i) => mons
            .get(*i)
            .ok_or_else(|| anyhow::anyhow!("No monitor with index {i} (detected {})", mons.len())),
        MonitorSelect::Name(n) => mons
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(n))
            .ok_or_else(|| anyhow::anyhow!("No monitor named '{n}'")),
    }
}

impl Capturer {
    pub fn new(select: &MonitorSelect) -> Result<Self> {
        let (conn, screen_num) = RustConnection::connect(None)?;
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        let bpp = conn
            .setup()
            .pixmap_formats
            .iter()
            .find(|f| f.depth == screen.root_depth)
            .map(|f| f.bits_per_pixel as usize / 8)
            .unwrap_or(4);

        // Resolve the requested monitor (defaults to the primary) via RandR
        let monitors = fetch_monitors(&conn, root)?;
        let mon = pick_monitor(&monitors, select)?;
        let mx = mon.x;
        let my = mon.y;
        let mw = mon.width;
        let mh = mon.height;
        eprintln!("Monitor: {} {}x{} at ({},{})", mon.name, mw, mh, mx, my);

        let bytes = mw * mh * bpp;

        let shm_id = unsafe { libc::shmget(IPC_PRIVATE, bytes, IPC_CREAT | 0o600) };
        if shm_id < 0 {
            bail!("shmget failed");
        }

        let shm_addr = unsafe { libc::shmat(shm_id, ptr::null(), 0) as *mut u8 };
        if shm_addr as isize == -1 {
            bail!("shmat failed");
        }

        // Mark for deletion — freed automatically when no more references
        unsafe {
            libc::shmctl(shm_id, IPC_RMID, ptr::null_mut());
        }

        let shm_seg = conn.generate_id()?;
        conn.shm_attach(shm_seg, shm_id as u32, false)?.check()?;

        Ok(Self {
            conn,
            root,
            mx,
            my,
            mw,
            mh,
            bpp,
            shm_addr,
            shm_seg,
        })
    }

    pub fn grab(&self, cfg: &Config) -> Result<Vec<(u8, u8, u8)>> {
        self.conn
            .shm_get_image(
                self.root,
                self.mx,
                self.my,
                self.mw as u16,
                self.mh as u16,
                !0u32,
                ImageFormat::Z_PIXMAP.into(),
                self.shm_seg,
                0,
            )?
            .reply()?;

        let cw = cfg.cap_w;
        let ch = cfg.cap_h;

        // Maps a (cap_y, cap_x) coordinate to a monitor pixel (BGR→RGB)
        let stride_y = self.mh / ch;
        let stride_x = self.mw / cw;
        let bpp = self.bpp;
        let mw = self.mw;

        let pixel = |py: usize, px: usize| -> (u8, u8, u8) {
            let off = (py * stride_y * mw + px * stride_x) * bpp;
            // X11 ZPixmap on little-endian: BGRX byte order
            let b = unsafe { *self.shm_addr.add(off) };
            let g = unsafe { *self.shm_addr.add(off + 1) };
            let r = unsafe { *self.shm_addr.add(off + 2) };
            (r, g, b)
        };

        // Average all pixels in a rectangle (cap-space coords)
        let avg = |x0: usize, x1: usize, y0: usize, y1: usize| -> (u8, u8, u8) {
            let (mut rs, mut gs, mut bs, mut n) = (0u32, 0u32, 0u32, 0u32);
            for py in y0..y1 {
                for px in x0..x1 {
                    let (r, g, b) = pixel(py, px);
                    rs += r as u32;
                    gs += g as u32;
                    bs += b as u32;
                    n += 1;
                }
            }
            if n == 0 {
                return (0, 0, 0);
            }
            ((rs / n) as u8, (gs / n) as u8, (bs / n) as u8)
        };

        let mut leds = Vec::with_capacity(cfg.leds());

        // Left (top → bottom)
        for i in 0..cfg.left {
            let y0 = i * ch / cfg.left;
            let y1 = ((i + 1) * ch / cfg.left).max(y0 + 1);
            let (r, g, b) = avg(0, cfg.border, y0, y1);
            leds.push(process(r, g, b, cfg));
        }

        // Top (left → right)
        for i in 0..cfg.top {
            let x0 = i * cw / cfg.top;
            let x1 = ((i + 1) * cw / cfg.top).max(x0 + 1);
            let (r, g, b) = avg(x0, x1, 0, cfg.border);
            leds.push(process(r, g, b, cfg));
        }

        // Right (top → bottom)
        for i in 0..cfg.right {
            let y0 = i * ch / cfg.right;
            let y1 = ((i + 1) * ch / cfg.right).max(y0 + 1);
            let (r, g, b) = avg(cw - cfg.border, cw, y0, y1);
            leds.push(process(r, g, b, cfg));
        }

        Ok(leds)
    }
}

impl Drop for Capturer {
    fn drop(&mut self) {
        let _ = self.conn.shm_detach(self.shm_seg);
        unsafe {
            libc::shmdt(self.shm_addr as *const _);
        }
    }
}

pub fn smooth(current: &[(u8, u8, u8)], prev: &[(u8, u8, u8)], factor: f32) -> Vec<(u8, u8, u8)> {
    current
        .iter()
        .zip(prev.iter())
        .map(|(&(r, g, b), &(pr, pg, pb))| {
            (
                (factor * pr as f32 + (1.0 - factor) * r as f32) as u8,
                (factor * pg as f32 + (1.0 - factor) * g as f32) as u8,
                (factor * pb as f32 + (1.0 - factor) * b as f32) as u8,
            )
        })
        .collect()
}

fn process(r: u8, g: u8, b: u8, cfg: &Config) -> (u8, u8, u8) {
    let mut rf = r as f32 / 255.0;
    let mut gf = g as f32 / 255.0;
    let mut bf = b as f32 / 255.0;

    let mx = rf.max(gf).max(bf);
    let mn = rf.min(gf).min(bf);

    if mx > 0.0 {
        let sat = (mx - mn) / mx;
        if sat > 0.0 {
            let scale = (sat * cfg.saturation).min(1.0) / sat;
            rf = (mx - (mx - rf) * scale).max(0.0);
            gf = (mx - (mx - gf) * scale).max(0.0);
            bf = (mx - (mx - bf) * scale).max(0.0);
        }
    }

    let inv_gamma = 1.0 / cfg.gamma;
    (
        (rf.powf(inv_gamma) * 255.0).min(255.0) as u8,
        (gf.powf(inv_gamma) * 255.0).min(255.0) as u8,
        (bf.powf(inv_gamma) * 255.0).min(255.0) as u8,
    )
}

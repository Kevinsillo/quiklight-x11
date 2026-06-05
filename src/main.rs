mod capture;
mod config;
mod device;
mod protocol;

use anyhow::Result;
use capture::MonitorSelect;
use clap::Parser;
use config::Config;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Ambilight driver for the Robobloq Quiklight LED strip (X11).
///
/// With no options it runs the capture loop using the config file. The flags
/// below override the config for this run; add --save to persist them.
#[derive(Parser, Debug)]
#[command(name = "quiklight", version, about, long_about = None)]
struct Cli {
    /// Capture this monitor instead of the primary one
    /// (index or name from `--list-monitors`, e.g. `1` or `DP-4`)
    #[arg(short = 'm', long, value_name = "INDEX|NAME")]
    monitor: Option<String>,

    /// Target frames per second
    #[arg(short = 'f', long, value_name = "N", value_parser = clap::value_parser!(u32).range(1..=240))]
    fps: Option<u32>,

    /// Capture resolution as WxH (see `--list-resolutions` for presets)
    #[arg(short = 'r', long, value_name = "WxH", value_parser = parse_resolution)]
    resolution: Option<(usize, usize)>,

    /// Global brightness sent to the device (0-255)
    #[arg(short = 'b', long, value_name = "0-255")]
    brightness: Option<u8>,

    /// Temporal smoothing: 0.0 = off … 0.95 = very smooth
    #[arg(short = 's', long, value_name = "F", value_parser = parse_smoothing)]
    smoothing: Option<f32>,

    /// Saturation boost: 1.0 = none, 2.0 = vivid
    #[arg(long, value_name = "F")]
    saturation: Option<f32>,

    /// Gamma correction: 1.0 = linear, >1 darkens midtones
    #[arg(long, value_name = "F")]
    gamma: Option<f32>,

    /// Border thickness sampled at each edge, in capture pixels
    #[arg(long, value_name = "PX")]
    border: Option<usize>,

    /// Number of LEDs across the top edge
    #[arg(long, value_name = "N")]
    top: Option<usize>,

    /// Number of LEDs on the right edge
    #[arg(long, value_name = "N")]
    right: Option<usize>,

    /// Number of LEDs on the left edge
    #[arg(long, value_name = "N")]
    left: Option<usize>,

    /// Use this config file instead of the default
    #[arg(short = 'c', long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// List detected monitors and exit
    #[arg(long)]
    list_monitors: bool,

    /// List capture-resolution presets for the selected monitor and exit
    #[arg(long)]
    list_resolutions: bool,

    /// Persist the resulting options to the config file, then keep running
    #[arg(long)]
    save: bool,

    /// Turn the LEDs off (send a single black frame) and exit
    #[arg(long)]
    off: bool,
}

impl Cli {
    /// Apply the options that were actually passed on top of the loaded config.
    fn apply(&self, cfg: &mut Config) {
        if let Some(v) = &self.monitor {
            cfg.monitor = Some(v.clone());
        }
        if let Some(v) = self.fps {
            cfg.fps = v;
        }
        if let Some((w, h)) = self.resolution {
            cfg.cap_w = w;
            cfg.cap_h = h;
        }
        if let Some(v) = self.brightness {
            cfg.brightness = v;
        }
        if let Some(v) = self.smoothing {
            cfg.smoothing = v;
        }
        if let Some(v) = self.saturation {
            cfg.saturation = v;
        }
        if let Some(v) = self.gamma {
            cfg.gamma = v;
        }
        if let Some(v) = self.border {
            cfg.border = v;
        }
        if let Some(v) = self.top {
            cfg.top = v;
        }
        if let Some(v) = self.right {
            cfg.right = v;
        }
        if let Some(v) = self.left {
            cfg.left = v;
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // --list-monitors works without a config file
    if cli.list_monitors {
        return print_monitors();
    }

    let config_path = cli.config.clone().unwrap_or_else(default_config_path);
    let mut cfg = Config::load(&config_path);
    cli.apply(&mut cfg);

    // --off doesn't capture the screen; it just blanks the strip and exits
    if cli.off {
        return turn_off(&cfg);
    }

    if cli.save {
        cfg.save(&config_path)?;
        eprintln!("Saved config → {}", config_path.display());
    }

    if cli.list_resolutions {
        return print_resolutions(&cfg);
    }

    run(&cfg)
}

fn run(cfg: &Config) -> Result<()> {
    let select = monitor_select(&cfg.monitor);
    let capturer = capture::Capturer::new(&select)?;
    let mut dev = device::open_device()?;
    let mut msg_id = protocol::init(&mut dev, cfg.brightness, cfg.leds())?;

    let interval = Duration::from_secs_f64(1.0 / cfg.fps as f64);
    let mut prev = vec![(0u8, 0u8, 0u8); cfg.leds()];

    eprintln!(
        "Quiklight running — {} LEDs @ {} fps | cap {}x{} | smoothing={} saturation={} gamma={}",
        cfg.leds(),
        cfg.fps,
        cfg.cap_w,
        cfg.cap_h,
        cfg.smoothing,
        cfg.saturation,
        cfg.gamma
    );

    loop {
        let t0 = Instant::now();

        let leds = capturer.grab(cfg)?;
        let leds = capture::smooth(&leds, &prev, cfg.smoothing);
        prev = leds.clone();

        protocol::send_frame(&mut dev, msg_id, &leds)?;
        msg_id = msg_id.wrapping_add(1);

        let elapsed = t0.elapsed();
        if elapsed < interval {
            std::thread::sleep(interval - elapsed);
        }
    }
}

/// Send a single all-black frame and exit (used as the service's ExecStop).
/// Does not touch X11, so it works even when no display is available.
fn turn_off(cfg: &Config) -> Result<()> {
    let mut dev = device::open_device()?;
    let msg_id = protocol::init(&mut dev, cfg.brightness, cfg.leds())?;
    let black = vec![(0u8, 0u8, 0u8); cfg.leds()];
    protocol::send_frame(&mut dev, msg_id, &black)?;
    eprintln!("LEDs off.");
    Ok(())
}

fn default_config_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_default()
        .join("quiklight-rust/config.toml")
}

fn monitor_select(m: &Option<String>) -> MonitorSelect {
    match m {
        None => MonitorSelect::Primary,
        Some(s) => match s.parse::<usize>() {
            Ok(i) => MonitorSelect::Index(i),
            Err(_) => MonitorSelect::Name(s.clone()),
        },
    }
}

fn print_monitors() -> Result<()> {
    let mons = capture::list_monitors()?;
    if mons.is_empty() {
        println!("No monitors detected.");
        return Ok(());
    }
    println!("Detected monitors (use the index or name with --monitor):");
    for m in &mons {
        let tag = if m.primary { "  *primary" } else { "" };
        println!(
            "  [{}] {:<12} {}x{} at ({},{}){}",
            m.index, m.name, m.width, m.height, m.x, m.y, tag
        );
    }
    Ok(())
}

fn print_resolutions(cfg: &Config) -> Result<()> {
    let mons = capture::list_monitors()?;
    let select = monitor_select(&cfg.monitor);
    let mon = capture::pick_monitor(&mons, &select)?;
    println!(
        "Capture-resolution presets for {} (native {}x{}):",
        mon.name, mon.width, mon.height
    );
    for (w, h) in resolution_presets(mon.width, mon.height) {
        let cur = if w == cfg.cap_w && h == cfg.cap_h {
            "   <- current"
        } else {
            ""
        };
        println!("  {:>5} x {:<5}{}", w, h, cur);
    }
    println!("\nUse e.g.:  quiklight --resolution 80x45");
    Ok(())
}

/// Integer-divisor resolutions of the native size, largest → smallest,
/// keeping the exact aspect ratio (always clean multiples).
fn resolution_presets(w: usize, h: usize) -> Vec<(usize, usize)> {
    let g = gcd(w, h);
    if g == 0 {
        return Vec::new();
    }
    let mut divisors: Vec<usize> = (1..=g).filter(|k| g % k == 0).collect();
    divisors.sort_unstable();
    divisors
        .into_iter()
        .map(|k| (w / k, h / k))
        .filter(|&(pw, _)| pw >= 16)
        .collect()
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn parse_resolution(s: &str) -> Result<(usize, usize), String> {
    let (w, h) = s
        .split_once(|c| c == 'x' || c == 'X')
        .ok_or_else(|| format!("expected WxH, got '{s}'"))?;
    let w = w
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("invalid width in '{s}'"))?;
    let h = h
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("invalid height in '{s}'"))?;
    if w == 0 || h == 0 {
        return Err("width and height must be > 0".into());
    }
    Ok((w, h))
}

fn parse_smoothing(s: &str) -> Result<f32, String> {
    let v: f32 = s.parse().map_err(|_| format!("'{s}' isn't a number"))?;
    if (0.0..=0.95).contains(&v) {
        Ok(v)
    } else {
        Err("smoothing must be between 0.0 and 0.95".into())
    }
}

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    // LED strip layout
    pub top: usize,
    pub right: usize,
    pub left: usize,
    // Screen capture
    pub cap_w: usize,
    pub cap_h: usize,
    pub border: usize,
    // Performance
    pub fps: u32,
    // Color processing
    pub brightness: u8,
    pub smoothing: f32,
    pub saturation: f32,
    pub gamma: f32,
    // Screen selection — None = primary monitor (index or name, e.g. "DP-4")
    #[serde(default)]
    pub monitor: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            top: 29,
            right: 17,
            left: 17,
            cap_w: 80,
            cap_h: 45,
            border: 3,
            fps: 60,
            brightness: 200,
            smoothing: 0.15,
            saturation: 1.4,
            gamma: 1.2,
            monitor: None,
        }
    }
}

impl Config {
    pub fn leds(&self) -> usize {
        self.top + self.right + self.left
    }

    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Write the config back to disk as a commented TOML file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let monitor_line = match &self.monitor {
            Some(m) => format!("monitor = \"{m}\""),
            None => "# monitor = \"DP-4\"   # omit to use the primary monitor".to_string(),
        };
        let content = format!(
            "# Quiklight — configuration (written by `quiklight --save`)\n\
             \n\
             # LED strip layout (Robobloq Quiklight default: 63 LEDs)\n\
             top   = {top}\n\
             right = {right}\n\
             left  = {left}\n\
             \n\
             # Screen capture resolution — lower = faster, less precise\n\
             cap_w  = {cap_w}\n\
             cap_h  = {cap_h}\n\
             border = {border}\n\
             \n\
             # Which monitor to capture\n\
             {monitor_line}\n\
             \n\
             # Performance\n\
             fps = {fps}\n\
             \n\
             # Color processing\n\
             brightness = {brightness}   # global brightness sent to device (0–255)\n\
             smoothing  = {smoothing}  # temporal smoothing (0.0 = off, 0.95 = very smooth)\n\
             saturation = {saturation}   # color saturation boost (1.0 = none, 2.0 = vivid)\n\
             gamma      = {gamma}   # gamma correction (1.0 = linear)\n",
            top = self.top,
            right = self.right,
            left = self.left,
            cap_w = self.cap_w,
            cap_h = self.cap_h,
            border = self.border,
            monitor_line = monitor_line,
            fps = self.fps,
            brightness = self.brightness,
            smoothing = self.smoothing,
            saturation = self.saturation,
            gamma = self.gamma,
        );
        std::fs::write(path, content)
    }
}

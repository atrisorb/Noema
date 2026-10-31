use std::io::IsTerminal;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub const fn hex(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }

    pub fn to_ansi_fg(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.0, self.1, self.2)
    }

    pub fn to_ansi_bg(&self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.0, self.1, self.2)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub artery: Rgb,
    pub nerve: Rgb,
    pub vein: Rgb,
    pub lymph: Rgb,
    pub danger: Rgb,
    pub warning: Rgb,
    pub ink: Rgb,
    pub ink_soft: Rgb,
    pub ink_muted: Rgb,
    pub surface: Rgb,
    pub surface_2: Rgb,
    pub bg: Rgb,
}

impl Theme {
    pub fn cavity() -> Self {
        Self {
            name: "cavity".into(),
            artery: Rgb::hex(255, 90, 95),     // #FF5A5F (coral crimson)
            nerve: Rgb::hex(255, 200, 87),     // #FFC857 (amber focus)
            vein: Rgb::hex(108, 140, 255),     // #6C8CFF (cobalt blue)
            lymph: Rgb::hex(125, 255, 196),    // #7DFFC4 (mint success)
            danger: Rgb::hex(255, 107, 107),   // #FF6B6B (destructive error)
            warning: Rgb::hex(255, 184, 107),  // #FFB86B (warm warning)
            ink: Rgb::hex(244, 239, 230),      // #F4EFE6 (warm bone)
            ink_soft: Rgb::hex(216, 210, 198), // #D8D2C6
            ink_muted: Rgb::hex(152, 162, 179),// #98A2B3
            surface: Rgb::hex(11, 17, 27),     // #0B111B
            surface_2: Rgb::hex(16, 24, 39),   // #101827
            bg: Rgb::hex(5, 7, 11),            // #05070B
        }
    }

    pub fn bone() -> Self {
        Self {
            name: "bone".into(),
            artery: Rgb::hex(199, 53, 63),     // #C7353F
            nerve: Rgb::hex(154, 106, 0),      // #9A6A00
            vein: Rgb::hex(63, 91, 214),       // #3F5BD6
            lymph: Rgb::hex(31, 122, 85),      // #1F7A55
            danger: Rgb::hex(179, 38, 30),     // #B3261E
            warning: Rgb::hex(138, 90, 0),     // #8A5A00
            ink: Rgb::hex(11, 13, 18),         // #0B0D12
            ink_soft: Rgb::hex(42, 47, 58),    // #2A2F3A
            ink_muted: Rgb::hex(92, 100, 114), // #5C6472
            surface: Rgb::hex(251, 248, 242),  // #FBF8F2
            surface_2: Rgb::hex(232, 223, 209),// #E8DFD1
            bg: Rgb::hex(244, 239, 230),       // #F4EFE6
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "nord".into(),
            artery: Rgb::hex(191, 97, 106),    // Nord11 red
            nerve: Rgb::hex(235, 203, 139),    // Nord13 yellow
            vein: Rgb::hex(129, 161, 193),     // Nord9 blue
            lymph: Rgb::hex(163, 190, 140),    // Nord14 green
            danger: Rgb::hex(191, 97, 106),
            warning: Rgb::hex(208, 135, 112),   // Nord12 orange
            ink: Rgb::hex(236, 239, 244),      // Nord6 snow
            ink_soft: Rgb::hex(229, 233, 240), // Nord5
            ink_muted: Rgb::hex(143, 188, 187),// Nord7
            surface: Rgb::hex(59, 66, 82),     // Nord1
            surface_2: Rgb::hex(67, 76, 94),   // Nord2
            bg: Rgb::hex(46, 52, 64),          // Nord0
        }
    }

    pub fn catppuccin() -> Self {
        Self {
            name: "catppuccin".into(),
            artery: Rgb::hex(243, 139, 168),   // Red
            nerve: Rgb::hex(249, 226, 175),    // Yellow
            vein: Rgb::hex(137, 180, 250),     // Blue
            lymph: Rgb::hex(166, 227, 161),    // Green
            danger: Rgb::hex(235, 160, 172),   // Maroon
            warning: Rgb::hex(250, 179, 135),  // Peach
            ink: Rgb::hex(205, 214, 244),      // Text
            ink_soft: Rgb::hex(186, 194, 222), // Subtext1
            ink_muted: Rgb::hex(147, 153, 178),// Overlay1
            surface: Rgb::hex(49, 50, 68),     // Surface0
            surface_2: Rgb::hex(69, 71, 90),   // Surface1
            bg: Rgb::hex(30, 30, 46),          // Base
        }
    }

    pub fn monokai() -> Self {
        Self {
            name: "monokai".into(),
            artery: Rgb::hex(255, 97, 136),    // Pink
            nerve: Rgb::hex(255, 216, 102),    // Yellow
            vein: Rgb::hex(120, 220, 232),     // Cyan
            lymph: Rgb::hex(169, 220, 103),    // Green
            danger: Rgb::hex(255, 97, 136),
            warning: Rgb::hex(252, 152, 103),  // Orange
            ink: Rgb::hex(252, 252, 250),
            ink_soft: Rgb::hex(193, 192, 192),
            ink_muted: Rgb::hex(114, 112, 114),
            surface: Rgb::hex(45, 42, 46),
            surface_2: Rgb::hex(64, 61, 65),
            bg: Rgb::hex(34, 31, 34),
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "bone" | "light" => Self::bone(),
            "nord" => Self::nord(),
            "catppuccin" | "mocha" => Self::catppuccin(),
            "monokai" => Self::monokai(),
            _ => Self::cavity(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub theme: String,
    pub no_color: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            theme: "cavity".into(),
            no_color: false,
        }
    }
}

pub fn config_path() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|h| PathBuf::from(h).join(".noema").join("cli_config.json"))
}

pub fn load_config() -> CliConfig {
    if let Some(path) = config_path() {
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(cfg) = serde_json::from_slice::<CliConfig>(&bytes) {
                return cfg;
            }
        }
    }
    CliConfig::default()
}

pub fn save_config(cfg: &CliConfig) -> Result<(), String> {
    if let Some(path) = config_path() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
        std::fs::write(&path, data).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn should_use_color(force_json: bool, force_no_color: bool) -> bool {
    if force_json || force_no_color {
        return false;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::io::stdout().is_terminal()
}

#[cfg(windows)]
unsafe extern "system" {
    fn GetStdHandle(nStdHandle: u32) -> isize;
    fn GetConsoleMode(hConsoleHandle: isize, lpMode: *mut u32) -> i32;
    fn SetConsoleMode(hConsoleHandle: isize, dwMode: u32) -> i32;
}

/// Enable Windows console Virtual Terminal Processing for ANSI 24-bit Truecolor.
pub fn init_terminal() {
    #[cfg(windows)]
    unsafe {
        const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5;
        const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle != 0 && handle != -1 {
            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) != 0 {
                SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

pub struct Styler {
    pub theme: Theme,
    pub active: bool,
}

impl Styler {
    pub fn new(force_json: bool, force_no_color: bool) -> Self {
        init_terminal();
        let cfg = load_config();
        let active = should_use_color(force_json, force_no_color || cfg.no_color);
        let theme = Theme::from_name(&cfg.theme);
        Self { theme, active }
    }

    pub fn color(&self, rgb: Rgb, s: &str) -> String {
        if !self.active {
            return s.to_string();
        }
        format!("{}{s}\x1b[0m", rgb.to_ansi_fg())
    }

    pub fn artery(&self, s: &str) -> String { self.color(self.theme.artery, s) }
    pub fn nerve(&self, s: &str) -> String { self.color(self.theme.nerve, s) }
    pub fn vein(&self, s: &str) -> String { self.color(self.theme.vein, s) }
    pub fn lymph(&self, s: &str) -> String { self.color(self.theme.lymph, s) }
    pub fn danger(&self, s: &str) -> String { self.color(self.theme.danger, s) }
    pub fn warning(&self, s: &str) -> String { self.color(self.theme.warning, s) }
    pub fn ink(&self, s: &str) -> String { self.color(self.theme.ink, s) }
    pub fn ink_soft(&self, s: &str) -> String { self.color(self.theme.ink_soft, s) }
    pub fn ink_muted(&self, s: &str) -> String { self.color(self.theme.ink_muted, s) }

    pub fn bold(&self, s: &str) -> String {
        if !self.active { return s.to_string(); }
        format!("\x1b[1m{s}\x1b[0m")
    }

    pub fn dim(&self, s: &str) -> String {
        if !self.active { return s.to_string(); }
        format!("\x1b[2m{s}\x1b[0m")
    }

    #[allow(dead_code)]
    pub fn badge(&self, label: &str, rgb: Rgb) -> String {
        if !self.active {
            return format!("[{label}]");
        }
        format!("{} {label} \x1b[0m", rgb.to_ansi_bg())
    }

    pub fn pass_badge(&self) -> String {
        if !self.active { return "[PASS]".to_string(); }
        format!("{}\x1b[30m PASS \x1b[0m", self.theme.lymph.to_ansi_bg())
    }

    pub fn warn_badge(&self) -> String {
        if !self.active { return "[WARN]".to_string(); }
        format!("{}\x1b[30m WARN \x1b[0m", self.theme.warning.to_ansi_bg())
    }

    pub fn fail_badge(&self) -> String {
        if !self.active { return "[FAIL]".to_string(); }
        format!("{}\x1b[30m FAIL \x1b[0m", self.theme.danger.to_ansi_bg())
    }

    /// Editorial ASCII wordmark and lockup.
    pub fn banner(&self, subtitle: Option<&str>) -> String {
        let n1 = " _   _    ___    _____  __  __     _   ";
        let n2 = "| \\ | |  / _ \\  | ____||  \\/  |   / \\  ";
        let n3 = "|  \\| | | | | | | |__  | \\  / |  / _ \\ ";
        let n4 = "| |\\  | | |_| | |  __| | |\\/| | / ___ \\";
        let n5 = "|_| \\_|  \\___/  |_____||_|  |_|/_/   \\_\\";

        if !self.active {
            let mut out = format!("{n1}\n{n2}\n{n3}\n{n4}\n{n5}\n");
            if let Some(sub) = subtitle {
                out.push_str(&format!("{sub}\n"));
            }
            return out;
        }

        let artery_fg = self.theme.artery.to_ansi_fg();
        let dim_fg = self.theme.ink_muted.to_ansi_fg();
        let sub_fg = self.theme.nerve.to_ansi_fg();
        let reset = "\x1b[0m";

        let mut lines = Vec::new();
        lines.push(format!("{artery_fg}{n1}{reset}"));
        lines.push(format!("{artery_fg}{n2}{reset}"));
        lines.push(format!("{artery_fg}{n3}{reset}  {dim_fg}the anatomical mind{reset}"));
        lines.push(format!("{artery_fg}{n4}{reset}  {sub_fg}v{}{reset}", noema_core::VERSION));
        lines.push(format!("{artery_fg}{n5}{reset}"));

        if let Some(sub) = subtitle {
            lines.push(format!("\n  {}", self.ink_soft(sub)));
        }

        lines.join("\n")
    }

    /// Render an elegant Vitreous boxed card.
    pub fn card(&self, title: &str, lines: &[String]) -> String {
        if !self.active {
            let mut out = format!("=== {title} ===\n");
            for l in lines {
                out.push_str(l);
                out.push('\n');
            }
            return out;
        }

        let max_content_len = lines
            .iter()
            .map(|l| strip_ansi(l).chars().count())
            .max()
            .unwrap_or(0);
        let width = max_content_len.max(strip_ansi(title).chars().count() + 6).max(56);

        let border_fg = self.theme.ink_muted.to_ansi_fg();
        let artery_fg = self.theme.artery.to_ansi_fg();
        let reset = "\x1b[0m";

        let mut out = String::new();
        let top_title_pad = width.saturating_sub(strip_ansi(title).chars().count() + 4);
        out.push_str(&format!(
            "{border_fg}╭─ {artery_fg}{title}{border_fg} {}╮{reset}\n",
            "─".repeat(top_title_pad)
        ));

        for line in lines {
            let visible_len = strip_ansi(line).chars().count();
            let pad = width.saturating_sub(visible_len);
            out.push_str(&format!(
                "{border_fg}│{reset}  {line}{}  {border_fg}│{reset}\n",
                " ".repeat(pad)
            ));
        }

        out.push_str(&format!("{border_fg}╰{}╯{reset}\n", "─".repeat(width + 4)));
        out
    }

    /// Render a stylish table.
    pub fn table(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        let mut col_widths = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    let visible_len = strip_ansi(cell).chars().count();
                    col_widths[i] = col_widths[i].max(visible_len);
                }
            }
        }

        let border_fg = if self.active { self.theme.ink_muted.to_ansi_fg() } else { "".into() };
        let header_fg = if self.active { self.theme.ink.to_ansi_fg() } else { "".into() };
        let reset = if self.active { "\x1b[0m" } else { "" };

        let mut out = String::new();

        // Top line
        out.push_str(&border_fg);
        out.push('┌');
        for (i, w) in col_widths.iter().enumerate() {
            out.push_str(&"─".repeat(*w + 2));
            if i + 1 < col_widths.len() { out.push('┬'); } else { out.push('┐'); }
        }
        out.push_str(reset);
        out.push('\n');

        // Header row
        out.push_str(&border_fg);
        out.push('│');
        for (i, h) in headers.iter().enumerate() {
            let pad = col_widths[i].saturating_sub(h.len());
            out.push_str(&format!(" {header_fg}{h}{reset}{} {border_fg}│", " ".repeat(pad)));
        }
        out.push_str(reset);
        out.push('\n');

        // Separator
        out.push_str(&border_fg);
        out.push('├');
        for (i, w) in col_widths.iter().enumerate() {
            out.push_str(&"─".repeat(*w + 2));
            if i + 1 < col_widths.len() { out.push('┼'); } else { out.push('┤'); }
        }
        out.push_str(reset);
        out.push('\n');

        // Rows
        for row in rows {
            out.push_str(&border_fg);
            out.push('│');
            for (i, cell) in row.iter().enumerate() {
                let visible = strip_ansi(cell).chars().count();
                let pad = col_widths.get(i).copied().unwrap_or(visible).saturating_sub(visible);
                out.push_str(&format!(" {cell}{} {border_fg}│", " ".repeat(pad)));
            }
            out.push_str(reset);
            out.push('\n');
        }

        // Bottom
        out.push_str(&border_fg);
        out.push('└');
        for (i, w) in col_widths.iter().enumerate() {
            out.push_str(&"─".repeat(*w + 2));
            if i + 1 < col_widths.len() { out.push('┴'); } else { out.push('┘'); }
        }
        out.push_str(reset);
        out.push('\n');

        out
    }
}

/// Strip ANSI escape codes to compute visible character count.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

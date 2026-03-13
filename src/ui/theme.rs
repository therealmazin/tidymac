use ratatui::style::Color;

// CPU / process colors
pub const CPU_GREEN: Color = Color::Rgb(102, 204, 102);

// Memory colors
pub const MEM_BLUE: Color = Color::Rgb(100, 160, 240);

// Disk colors
pub const DISK_MAGENTA: Color = Color::Rgb(180, 120, 220);

// Status colors
pub const WARN_YELLOW: Color = Color::Rgb(230, 200, 80);
pub const CRIT_RED: Color = Color::Rgb(230, 90, 90);

// UI accent (highlights, selection)
pub const ACCENT: Color = Color::Rgb(100, 180, 230);

// Text
pub const TEXT_PRIMARY: Color = Color::White;
pub const TEXT_SECONDARY: Color = Color::Rgb(140, 140, 140);

// Borders
pub const BORDER_FOCUSED: Color = Color::Rgb(100, 180, 230);
pub const BORDER_NORMAL: Color = Color::Rgb(80, 80, 80);

// Bars / backgrounds
pub const BG_BAR: Color = Color::Rgb(50, 50, 50);
pub const SELECTED_BG: Color = Color::Rgb(40, 50, 65);

// Spinner
pub const SPINNER_COLOR: Color = Color::Rgb(100, 180, 230);

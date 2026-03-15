use ratatui::style::Color;

// Catppuccin Mocha palette
// https://github.com/catppuccin/catppuccin

// CPU / process colors
pub const CPU_GREEN: Color = Color::Rgb(166, 227, 161);     // Green

// Memory colors
pub const MEM_BLUE: Color = Color::Rgb(137, 180, 250);      // Blue

// Disk colors
pub const DISK_MAGENTA: Color = Color::Rgb(203, 166, 247);   // Mauve

// Status colors
pub const WARN_YELLOW: Color = Color::Rgb(249, 226, 175);    // Yellow
pub const CRIT_RED: Color = Color::Rgb(243, 139, 168);       // Red

// UI accent (highlights, selection)
pub const ACCENT: Color = Color::Rgb(137, 180, 250);         // Blue

// Text
pub const TEXT_PRIMARY: Color = Color::Rgb(205, 214, 244);    // Text
pub const TEXT_SECONDARY: Color = Color::Rgb(147, 153, 178);  // Overlay1

// Borders
pub const BORDER_FOCUSED: Color = Color::Rgb(137, 180, 250); // Blue
pub const BORDER_NORMAL: Color = Color::Rgb(88, 91, 112);    // Surface2

// Bars / backgrounds
pub const BG_BAR: Color = Color::Rgb(49, 50, 68);            // Surface0
pub const SELECTED_BG: Color = Color::Rgb(69, 71, 90);       // Surface1

// Spinner
pub const SPINNER_COLOR: Color = Color::Rgb(180, 190, 254);  // Lavender

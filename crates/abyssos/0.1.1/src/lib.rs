//! Abyssos terminal colour theme.
//!
//! Very dark navy/charcoal background, teal/cyan hero accent.
//! Supports Ghostty and Blink Shell.
//!
//! # Palette
//!
//! ```rust
//! use abyssos::CYAN;
//! assert_eq!(CYAN, "#20C5C0");
//! ```

pub const BACKGROUND:     &str = "#0D0F10";
pub const FOREGROUND:     &str = "#E2E4E6";
pub const CURSOR:         &str = "#20C5C0";
pub const SELECTION_BG:   &str = "#1A3535";
pub const SELECTION_FG:   &str = "#E2E4E6";

// Normal colours
pub const BLACK:          &str = "#1A1C1E";
pub const RED:            &str = "#E06B6B";
pub const GREEN:          &str = "#1CB8A8";
pub const YELLOW:         &str = "#D4A85A";
pub const BLUE:           &str = "#5B8DD9";
pub const MAGENTA:        &str = "#9B7FD4";
pub const CYAN:           &str = "#20C5C0";
pub const WHITE:          &str = "#C8CACC";

// Bright colours
pub const BRIGHT_BLACK:   &str = "#2A2C2E";
pub const BRIGHT_RED:     &str = "#EA7B7B";
pub const BRIGHT_GREEN:   &str = "#2CC8B8";
pub const BRIGHT_YELLOW:  &str = "#E4B86A";
pub const BRIGHT_BLUE:    &str = "#6B9DE9";
pub const BRIGHT_MAGENTA: &str = "#AB8FE4";
pub const BRIGHT_CYAN:    &str = "#30D5D0";
pub const BRIGHT_WHITE:   &str = "#D8DADC";

/// Full 16-colour palette in ANSI order (0–15).
pub const PALETTE: [&str; 16] = [
    BLACK, RED, GREEN, YELLOW,
    BLUE, MAGENTA, CYAN, WHITE,
    BRIGHT_BLACK, BRIGHT_RED, BRIGHT_GREEN, BRIGHT_YELLOW,
    BRIGHT_BLUE, BRIGHT_MAGENTA, BRIGHT_CYAN, BRIGHT_WHITE,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hero_accent_is_teal() {
        assert_eq!(CYAN, "#20C5C0");
        assert_eq!(CURSOR, CYAN);
    }

    #[test]
    fn palette_has_sixteen_entries() {
        assert_eq!(PALETTE.len(), 16);
    }
}

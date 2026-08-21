"""
Abyssos — terminal colour theme.

Very dark navy/charcoal background, teal/cyan hero accent.
Supports Ghostty and Blink Shell.

Usage::

    import abyssos
    print(abyssos.PALETTE["cyan"])  # '#20C5C0'
"""

PALETTE = {
    "background":      "#0D0F10",
    "foreground":      "#E2E4E6",
    "cursor":          "#20C5C0",
    "selection_bg":    "#1A3535",
    "selection_fg":    "#E2E4E6",
    # Normal
    "black":           "#1A1C1E",
    "red":             "#E06B6B",
    "green":           "#1CB8A8",
    "yellow":          "#D4A85A",
    "blue":            "#5B8DD9",
    "magenta":         "#9B7FD4",
    "cyan":            "#20C5C0",
    "white":           "#C8CACC",
    # Bright
    "bright_black":    "#2A2C2E",
    "bright_red":      "#EA7B7B",
    "bright_green":    "#2CC8B8",
    "bright_yellow":   "#E4B86A",
    "bright_blue":     "#6B9DE9",
    "bright_magenta":  "#AB8FE4",
    "bright_cyan":     "#30D5D0",
    "bright_white":    "#D8DADC",
}

__version__ = "0.1.0"
__all__ = ["PALETTE"]

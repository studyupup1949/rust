// Abyssos — terminal colour theme for Blink Shell
// Very dark navy/charcoal background, teal/cyan hero accent.
// Install: Blink Settings → Appearance → New Theme → paste raw GitHub URL
// https://github.com/terry-li-hm/abyssos

(function(t) {
  t.prefs_.set('background-color', '#0D0F10');
  t.prefs_.set('foreground-color', '#E2E4E6');
  t.prefs_.set('cursor-color', '#20C5C0');

  t.prefs_.set('color-palette-overrides', [
    '#1A1C1E', // 0  black
    '#E06B6B', // 1  red
    '#1CB8A8', // 2  green
    '#D4A85A', // 3  yellow
    '#5B8DD9', // 4  blue
    '#9B7FD4', // 5  magenta
    '#20C5C0', // 6  cyan (hero)
    '#C8CACC', // 7  white
    '#2A2C2E', // 8  bright black
    '#EA7B7B', // 9  bright red
    '#2CC8B8', // 10 bright green
    '#E4B86A', // 11 bright yellow
    '#6B9DE9', // 12 bright blue
    '#AB8FE4', // 13 bright magenta
    '#30D5D0', // 14 bright cyan
    '#D8DADC', // 15 bright white
  ]);
})(term_);

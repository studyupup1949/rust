# Font -- Pure Rust font parsers

## Supported Formats

### CFF (Compact Font Format)
 - Charstring format 1 and 2 are fully implemented
 - All glyphs that are listed by `/Encoding` can be accessed via `gid_for_codepoint`. Glyphs can be looked up from unicode values if they are defined in Adobes `StandardEncoding`.
 - All glyphs can be accessed by name using `gid_for_name`

### Type1
 - Contains a PostScript interpreter (without file access)
 - Calling PostScript from CharStrings (used for Hinting) is not implemented. Instead they are emulated and the correct outline is produced.
 - Glyphs can accessed with:
   - `gid_for_name` using the name of the charstring
   - `gid_for_codepoint` using the built in `/Encoding`
   - `gid_for_unicode_codepoint` using the [AFL-Glyphlist](https://github.com/adobe-type-tools/agl-aglfn)

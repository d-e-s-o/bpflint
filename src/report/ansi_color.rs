use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Deref;
use std::slice;


/// Parse hex digit to value.
const fn parse_hex_digit(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        _ => panic!("Invalid hex digit"),
    }
}

/// Parse two hex digits into a byte value
const fn parse_hex_byte(high: u8, low: u8) -> u8 {
    parse_hex_digit(high) * 16 + parse_hex_digit(low)
}

/// A type that holds an ANSI color sequence.
pub(crate) struct AnsiColorStr {
    bytes: [u8; 20],
    len: usize,
}

impl AnsiColorStr {
    /// Get the ANSI sequence as a string slice.
    pub const fn as_str(&self) -> &str {
        // We need to work around the const slicing limitation
        // Create a slice using ptr operations which are const-stable
        unsafe {
            let slice = slice::from_raw_parts(self.bytes.as_ptr(), self.len);
            str::from_utf8_unchecked(slice)
        }
    }
}

impl Deref for AnsiColorStr {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Display for AnsiColorStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(self.as_str())
    }
}

/// Convert hex color to ANSI escape sequence at compile time.
pub(crate) const fn hex_color_to_ansi(color: &str) -> AnsiColorStr {
    let bytes = color.as_bytes();

    // Skip '#' if present and get the hex part.
    let hex_start = if !bytes.is_empty() && bytes[0] == b'#' {
        1
    } else {
        0
    };

    // Manually extract the 6 hex digits
    if bytes.len() != hex_start + 6 {
        panic!("Color must be exactly 6 hex digits");
    }

    let r_h = bytes[hex_start];
    let r_l = bytes[hex_start + 1];
    let g_h = bytes[hex_start + 2];
    let g_l = bytes[hex_start + 3];
    let b_h = bytes[hex_start + 4];
    let b_l = bytes[hex_start + 5];

    // Parse RGB values.
    let r = parse_hex_byte(r_h, r_l);
    let g = parse_hex_byte(g_h, g_l);
    let b = parse_hex_byte(b_h, b_l);

    // Convert RGB values to decimal strings manually.
    let r_hundreds = r / 100;
    let r_tens = (r % 100) / 10;
    let r_ones = r % 10;

    let g_hundreds = g / 100;
    let g_tens = (g % 100) / 10;
    let g_ones = g % 10;

    let b_hundreds = b / 100;
    let b_tens = (b % 100) / 10;
    let b_ones = b % 10;

    // Build the ANSI sequence: "\x1b[38;2;R;G;Bm".
    let mut result = [0u8; 20];
    let mut pos = 0;

    // "\x1b[38;2;"
    result[pos] = 0x1b;
    pos += 1;
    result[pos] = b'[';
    pos += 1;
    result[pos] = b'3';
    pos += 1;
    result[pos] = b'8';
    pos += 1;
    result[pos] = b';';
    pos += 1;
    result[pos] = b'2';
    pos += 1;
    result[pos] = b';';
    pos += 1;

    // Red value
    if r_hundreds > 0 {
        result[pos] = b'0' + r_hundreds;
        pos += 1;
    }
    if r_hundreds > 0 || r_tens > 0 {
        result[pos] = b'0' + r_tens;
        pos += 1;
    }
    result[pos] = b'0' + r_ones;
    pos += 1;
    result[pos] = b';';
    pos += 1;

    // Green value
    if g_hundreds > 0 {
        result[pos] = b'0' + g_hundreds;
        pos += 1;
    }
    if g_hundreds > 0 || g_tens > 0 {
        result[pos] = b'0' + g_tens;
        pos += 1;
    }
    result[pos] = b'0' + g_ones;
    pos += 1;
    result[pos] = b';';
    pos += 1;

    // Blue value
    if b_hundreds > 0 {
        result[pos] = b'0' + b_hundreds;
        pos += 1;
    }
    if b_hundreds > 0 || b_tens > 0 {
        result[pos] = b'0' + b_tens;
        pos += 1;
    }
    result[pos] = b'0' + b_ones;
    pos += 1;

    result[pos] = b'm';
    pos += 1;

    AnsiColorStr {
        bytes: result,
        len: pos,
    }
}


/// A macro to create a ANSI color sequence based on an HTML style color
/// specification.
macro_rules! AnsiColor {
    ($color:expr) => {{
        const PARSED: $crate::report::ansi_color::AnsiColorStr =
            $crate::report::ansi_color::hex_color_to_ansi($color);
        PARSED.as_str()
    }};
}


pub(crate) const COLOR_PURPLE: &str = AnsiColor!("#795da3");
pub(crate) const COLOR_TEAL: &str = AnsiColor!("#0086b3");
pub(crate) const COLOR_PINK: &str = AnsiColor!("#a71d5d");
pub(crate) const COLOR_INDIGO: &str = AnsiColor!("#183691");
pub(crate) const COLOR_GRAY: &str = AnsiColor!("#969896");
pub(crate) const COLOR_DARKGRAY: &str = AnsiColor!("#333333");
pub(crate) const COLOR_RESET: &str = "\x1b[0m";


#[cfg(test)]
mod tests {
    /// Check that the `AnsiColor` macro emits the expected color
    /// sequence strings.
    #[test]
    fn color_strings() {
        const RED: &str = AnsiColor!("#ff0000");
        const GREEN: &str = AnsiColor!("#00ff00");
        const BLUE: &str = AnsiColor!("#0000ff");

        assert_eq!(RED, "\x1b[38;2;255;0;0m");
        assert_eq!(GREEN, "\x1b[38;2;0;255;0m");
        assert_eq!(BLUE, "\x1b[38;2;0;0;255m");

        assert_eq!(AnsiColor!("#795da3"), "\x1b[38;2;121;93;163m");
    }
}

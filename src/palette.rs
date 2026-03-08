//! Core color type and the Nord palette.
//!
//! Provides an sRGB [`Color`] struct with hex parsing, GPU-ready float
//! conversion, gamma-correct linear transforms, and interpolation.
//! The full [Nord](https://www.nordtheme.com/) palette is exposed as the
//! [`NORD`] constant, organised into four groups via [`NordPalette`].

use std::fmt;

use serde::{Deserialize, Serialize};

/// An sRGB color with 8-bit channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    /// Red channel (0..=255).
    pub r: u8,
    /// Green channel (0..=255).
    pub g: u8,
    /// Blue channel (0..=255).
    pub b: u8,
}

impl Color {
    /// Creates a new color from 8-bit RGB components.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parses a hex color string (with or without leading `#`).
    ///
    /// Accepts `"#2E3440"` or `"2E3440"` (case-insensitive, 6 hex digits).
    pub fn from_hex(hex: &str) -> Result<Self, HexParseError> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        if hex.len() != 6 {
            return Err(HexParseError::InvalidLength(hex.len()));
        }
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| HexParseError::InvalidChar)?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| HexParseError::InvalidChar)?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| HexParseError::InvalidChar)?;
        Ok(Self { r, g, b })
    }

    /// Formats the color as an uppercase hex string with leading `#`.
    #[must_use]
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Converts to a float RGB triple in `[0.0, 1.0]` (sRGB space).
    ///
    /// Suitable for GPU uniform buffers and shader inputs that expect sRGB.
    #[must_use]
    pub fn to_rgb_f32(&self) -> [f32; 3] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
        ]
    }

    /// Constructs a color from float RGB components in `[0.0, 1.0]`.
    ///
    /// Values are clamped to the valid range before quantising to `u8`.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn from_rgb_f32(rgb: [f32; 3]) -> Self {
        Self {
            r: (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
            g: (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
            b: (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        }
    }

    /// Converts from sRGB to linear RGB (gamma decode).
    ///
    /// Uses the standard sRGB transfer function: values below the linear
    /// threshold use a simple scale, above use the power curve with
    /// exponent ~2.4.
    #[must_use]
    pub fn to_linear(&self) -> [f32; 3] {
        [
            srgb_to_linear(f32::from(self.r) / 255.0),
            srgb_to_linear(f32::from(self.g) / 255.0),
            srgb_to_linear(f32::from(self.b) / 255.0),
        ]
    }

    /// Constructs a `Color` from linear-space float components by applying
    /// the inverse sRGB transfer function (gamma encode) and quantising
    /// to `u8`.
    #[must_use]
    pub fn from_linear(linear: [f32; 3]) -> Self {
        Self::from_rgb_f32([
            linear_to_srgb(linear[0]),
            linear_to_srgb(linear[1]),
            linear_to_srgb(linear[2]),
        ])
    }

    /// Linearly interpolates between `self` and `other` in sRGB space.
    ///
    /// `t` is clamped to `[0.0, 1.0]`. At `t = 0.0` the result is `self`,
    /// at `t = 1.0` the result is `other`.
    #[must_use]
    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let a = self.to_rgb_f32();
        let b = other.to_rgb_f32();
        Color::from_rgb_f32([
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ])
    }

    /// Returns an RGBA float array with the given alpha.
    ///
    /// The RGB channels are in sRGB `[0.0, 1.0]` space; `alpha` is clamped
    /// to `[0.0, 1.0]`.
    #[must_use]
    pub fn with_alpha(&self, alpha: f32) -> [f32; 4] {
        let rgb = self.to_rgb_f32();
        [rgb[0], rgb[1], rgb[2], alpha.clamp(0.0, 1.0)]
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

// ── sRGB transfer functions ────────────────────────────────────────────

/// sRGB to linear conversion for a single component.
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear to sRGB conversion for a single component.
fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

// ── Hex parse error ────────────────────────────────────────────────────

/// Errors returned when parsing a hex color string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexParseError {
    /// The string (after stripping `#`) was not exactly 6 characters.
    InvalidLength(usize),
    /// A character was not a valid hexadecimal digit.
    InvalidChar,
}

impl fmt::Display for HexParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(n) => {
                write!(f, "expected 6 hex digits, got {n}")
            }
            Self::InvalidChar => write!(f, "invalid hex character"),
        }
    }
}

impl std::error::Error for HexParseError {}

// ── Nord palette ───────────────────────────────────────────────────────

/// The complete Nord color palette, organised into its four named groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NordPalette {
    /// Polar Night (nord0..nord3) — dark background tones.
    pub polar_night: [Color; 4],
    /// Snow Storm (nord4..nord6) — light foreground tones.
    pub snow_storm: [Color; 3],
    /// Frost (nord7..nord10) — blue accent tones.
    pub frost: [Color; 4],
    /// Aurora (nord11..nord15) — status / accent colors.
    pub aurora: [Color; 5],
}

/// The canonical Nord palette.
///
/// Values from <https://www.nordtheme.com/docs/colors-and-palettes>.
pub const NORD: NordPalette = NordPalette {
    polar_night: [
        Color::new(0x2E, 0x34, 0x40), // nord0
        Color::new(0x3B, 0x42, 0x52), // nord1
        Color::new(0x43, 0x4C, 0x5E), // nord2
        Color::new(0x4C, 0x56, 0x6A), // nord3
    ],
    snow_storm: [
        Color::new(0xD8, 0xDE, 0xE9), // nord4
        Color::new(0xE5, 0xE9, 0xF0), // nord5
        Color::new(0xEC, 0xEF, 0xF4), // nord6
    ],
    frost: [
        Color::new(0x8F, 0xBC, 0xBB), // nord7
        Color::new(0x88, 0xC0, 0xD0), // nord8
        Color::new(0x81, 0xA1, 0xC1), // nord9
        Color::new(0x5E, 0x81, 0xAC), // nord10
    ],
    aurora: [
        Color::new(0xBF, 0x61, 0x6A), // nord11 — red
        Color::new(0xD0, 0x87, 0x70), // nord12 — orange
        Color::new(0xEB, 0xCB, 0x8B), // nord13 — yellow
        Color::new(0xA3, 0xBE, 0x8C), // nord14 — green
        Color::new(0xB4, 0x8E, 0xAD), // nord15 — purple
    ],
};

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction ---------------------------------------------------

    #[test]
    fn color_new() {
        let c = Color::new(0x2E, 0x34, 0x40);
        assert_eq!(c.r, 0x2E);
        assert_eq!(c.g, 0x34);
        assert_eq!(c.b, 0x40);
    }

    // -- Hex parsing ----------------------------------------------------

    #[test]
    fn from_hex_with_hash() {
        let c = Color::from_hex("#2E3440").unwrap();
        assert_eq!(c, Color::new(0x2E, 0x34, 0x40));
    }

    #[test]
    fn from_hex_without_hash() {
        let c = Color::from_hex("2E3440").unwrap();
        assert_eq!(c, Color::new(0x2E, 0x34, 0x40));
    }

    #[test]
    fn from_hex_lowercase() {
        let c = Color::from_hex("#2e3440").unwrap();
        assert_eq!(c, Color::new(0x2E, 0x34, 0x40));
    }

    #[test]
    fn to_hex_roundtrip() {
        let original = "#88C0D0";
        let c = Color::from_hex(original).unwrap();
        assert_eq!(c.to_hex(), original);
    }

    #[test]
    fn from_hex_invalid_length_short() {
        let err = Color::from_hex("#FFF").unwrap_err();
        assert_eq!(err, HexParseError::InvalidLength(3));
    }

    #[test]
    fn from_hex_invalid_length_long() {
        let err = Color::from_hex("#11223344").unwrap_err();
        assert_eq!(err, HexParseError::InvalidLength(8));
    }

    #[test]
    fn from_hex_invalid_chars() {
        let err = Color::from_hex("#GGHHII").unwrap_err();
        assert_eq!(err, HexParseError::InvalidChar);
    }

    #[test]
    fn from_hex_empty_string() {
        let err = Color::from_hex("").unwrap_err();
        assert_eq!(err, HexParseError::InvalidLength(0));
    }

    // -- Float conversion -----------------------------------------------

    #[test]
    fn to_rgb_f32_black() {
        assert_eq!(Color::new(0, 0, 0).to_rgb_f32(), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn to_rgb_f32_white() {
        assert_eq!(Color::new(255, 255, 255).to_rgb_f32(), [1.0, 1.0, 1.0]);
    }

    #[test]
    fn rgb_f32_roundtrip() {
        let c = Color::new(0x88, 0xC0, 0xD0);
        let floats = c.to_rgb_f32();
        let back = Color::from_rgb_f32(floats);
        assert_eq!(c, back);
    }

    #[test]
    fn from_rgb_f32_clamps() {
        let c = Color::from_rgb_f32([-0.5, 1.5, 0.5]);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 128);
    }

    // -- Gamma conversion -----------------------------------------------

    #[test]
    fn to_linear_black_is_zero() {
        let lin = Color::new(0, 0, 0).to_linear();
        assert_eq!(lin, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn to_linear_white_is_one() {
        let lin = Color::new(255, 255, 255).to_linear();
        for ch in lin {
            assert!((ch - 1.0).abs() < 1e-5, "expected ~1.0, got {ch}");
        }
    }

    #[test]
    fn linear_srgb_roundtrip() {
        let original = Color::new(0x88, 0xC0, 0xD0);
        let linear = original.to_linear();
        let back = Color::from_linear(linear);
        assert_eq!(original, back);
    }

    #[test]
    fn to_linear_midgrey() {
        // sRGB 128 should map to ~0.216 in linear (not 0.5)
        let lin = Color::new(128, 128, 128).to_linear();
        assert!(
            (lin[0] - 0.216).abs() < 0.01,
            "expected ~0.216, got {}",
            lin[0]
        );
    }

    // -- Lerp -----------------------------------------------------------

    #[test]
    fn lerp_at_zero() {
        let a = Color::new(0, 0, 0);
        let b = Color::new(255, 255, 255);
        assert_eq!(a.lerp(&b, 0.0), a);
    }

    #[test]
    fn lerp_at_one() {
        let a = Color::new(0, 0, 0);
        let b = Color::new(255, 255, 255);
        assert_eq!(a.lerp(&b, 1.0), b);
    }

    #[test]
    fn lerp_at_half() {
        let a = Color::new(0, 0, 0);
        let b = Color::new(254, 254, 254);
        let mid = a.lerp(&b, 0.5);
        // 254 * 0.5 = 127
        assert_eq!(mid, Color::new(127, 127, 127));
    }

    // -- with_alpha -----------------------------------------------------

    #[test]
    fn with_alpha_returns_rgba() {
        let c = Color::new(255, 0, 0);
        let rgba = c.with_alpha(0.5);
        assert_eq!(rgba, [1.0, 0.0, 0.0, 0.5]);
    }

    #[test]
    fn with_alpha_clamps() {
        let c = Color::new(0, 0, 0);
        let rgba = c.with_alpha(2.0);
        assert_eq!(rgba[3], 1.0);
        let rgba_neg = c.with_alpha(-1.0);
        assert_eq!(rgba_neg[3], 0.0);
    }

    // -- Nord palette ---------------------------------------------------

    #[test]
    fn nord_polar_night_hex_values() {
        let expected = ["#2E3440", "#3B4252", "#434C5E", "#4C566A"];
        for (i, hex) in expected.iter().enumerate() {
            assert_eq!(NORD.polar_night[i].to_hex(), *hex);
        }
    }

    #[test]
    fn nord_snow_storm_hex_values() {
        let expected = ["#D8DEE9", "#E5E9F0", "#ECEFF4"];
        for (i, hex) in expected.iter().enumerate() {
            assert_eq!(NORD.snow_storm[i].to_hex(), *hex);
        }
    }

    #[test]
    fn nord_frost_hex_values() {
        let expected = ["#8FBCBB", "#88C0D0", "#81A1C1", "#5E81AC"];
        for (i, hex) in expected.iter().enumerate() {
            assert_eq!(NORD.frost[i].to_hex(), *hex);
        }
    }

    #[test]
    fn nord_aurora_hex_values() {
        let expected = ["#BF616A", "#D08770", "#EBCB8B", "#A3BE8C", "#B48EAD"];
        for (i, hex) in expected.iter().enumerate() {
            assert_eq!(NORD.aurora[i].to_hex(), *hex);
        }
    }

    #[test]
    fn nord_palette_array_lengths() {
        assert_eq!(NORD.polar_night.len(), 4);
        assert_eq!(NORD.snow_storm.len(), 3);
        assert_eq!(NORD.frost.len(), 4);
        assert_eq!(NORD.aurora.len(), 5);
    }

    // -- Serde ----------------------------------------------------------

    #[test]
    fn serde_color_roundtrip() {
        let c = Color::new(0x88, 0xC0, 0xD0);
        let json = serde_json::to_string(&c).unwrap();
        let back: Color = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn serde_nord_palette_roundtrip() {
        let json = serde_json::to_string(&NORD).unwrap();
        let back: NordPalette = serde_json::from_str(&json).unwrap();
        assert_eq!(NORD, back);
    }

    // -- Display --------------------------------------------------------

    #[test]
    fn display_matches_to_hex() {
        let c = Color::new(0xBF, 0x61, 0x6A);
        assert_eq!(format!("{c}"), c.to_hex());
    }
}

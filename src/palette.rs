//! Core color type and the Nord palette.
//!
//! Provides an sRGB [`Color`] struct with hex parsing, GPU-ready float
//! conversion, gamma-correct linear transforms, and interpolation.
//! The full [Nord](https://www.nordtheme.com/) palette is exposed as the
//! [`NORD`] constant, organised into four groups via [`NordPalette`].

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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

impl FromStr for Color {
    type Err = HexParseError;

    /// Parses a color from a hex string, delegating to [`Color::from_hex`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
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
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum HexParseError {
    /// The string (after stripping `#`) was not exactly 6 characters.
    #[error("expected 6 hex digits, got {0}")]
    InvalidLength(usize),
    /// A character was not a valid hexadecimal digit.
    #[error("invalid hex character")]
    InvalidChar,
}

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

impl NordPalette {
    /// Returns all 16 palette colors in Nord index order (nord0..nord15).
    #[must_use]
    pub fn all_colors(&self) -> [Color; 16] {
        let mut out = [Color::new(0, 0, 0); 16];
        let mut i = 0;
        for &c in &self.polar_night {
            out[i] = c;
            i += 1;
        }
        for &c in &self.snow_storm {
            out[i] = c;
            i += 1;
        }
        for &c in &self.frost {
            out[i] = c;
            i += 1;
        }
        for &c in &self.aurora {
            out[i] = c;
            i += 1;
        }
        out
    }
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
#[allow(clippy::float_cmp)]
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

    // -- Hex parsing: additional edge cases --------------------------------

    #[test]
    fn from_hex_hash_only() {
        let err = Color::from_hex("#").unwrap_err();
        assert_eq!(err, HexParseError::InvalidLength(0));
    }

    #[test]
    fn from_hex_all_zeros() {
        let c = Color::from_hex("#000000").unwrap();
        assert_eq!(c, Color::new(0, 0, 0));
    }

    #[test]
    fn from_hex_all_ffs() {
        let c = Color::from_hex("#FFFFFF").unwrap();
        assert_eq!(c, Color::new(255, 255, 255));
    }

    #[test]
    fn from_hex_mixed_case() {
        let c = Color::from_hex("#aAbBcC").unwrap();
        assert_eq!(c, Color::new(0xAA, 0xBB, 0xCC));
    }

    #[test]
    fn from_hex_five_digits() {
        let err = Color::from_hex("12345").unwrap_err();
        assert_eq!(err, HexParseError::InvalidLength(5));
    }

    #[test]
    fn from_hex_seven_digits_no_hash() {
        let err = Color::from_hex("1234567").unwrap_err();
        assert_eq!(err, HexParseError::InvalidLength(7));
    }

    #[test]
    fn from_hex_invalid_in_green_channel() {
        let err = Color::from_hex("#FFZZ00").unwrap_err();
        assert_eq!(err, HexParseError::InvalidChar);
    }

    #[test]
    fn from_hex_invalid_in_blue_channel() {
        let err = Color::from_hex("#FF00GG").unwrap_err();
        assert_eq!(err, HexParseError::InvalidChar);
    }

    #[test]
    fn to_hex_preserves_leading_zeros() {
        let c = Color::new(0x01, 0x02, 0x03);
        assert_eq!(c.to_hex(), "#010203");
    }

    // -- HexParseError Display ---------------------------------------------

    #[test]
    fn hex_parse_error_display_invalid_length() {
        let err = HexParseError::InvalidLength(3);
        assert_eq!(format!("{err}"), "expected 6 hex digits, got 3");
    }

    #[test]
    fn hex_parse_error_display_invalid_char() {
        let err = HexParseError::InvalidChar;
        assert_eq!(format!("{err}"), "invalid hex character");
    }

    #[test]
    fn hex_parse_error_is_error_trait() {
        let err: Box<dyn std::error::Error> =
            Box::new(HexParseError::InvalidChar);
        assert_eq!(format!("{err}"), "invalid hex character");
    }

    // -- sRGB/linear transfer function precision ---------------------------

    #[test]
    fn srgb_to_linear_below_threshold() {
        // For sRGB value 0.04045, this is the boundary of the linear region.
        // Below threshold: linear = sRGB / 12.92
        let val = 0.04045_f32;
        let lin = srgb_to_linear(val);
        let expected = val / 12.92;
        assert!(
            (lin - expected).abs() < 1e-7,
            "below threshold: expected {expected}, got {lin}"
        );
    }

    #[test]
    fn srgb_to_linear_above_threshold() {
        // 0.5 is well above the threshold; use the power curve.
        let lin = srgb_to_linear(0.5);
        let expected = ((0.5 + 0.055) / 1.055_f32).powf(2.4);
        assert!(
            (lin - expected).abs() < 1e-7,
            "above threshold: expected {expected}, got {lin}"
        );
    }

    #[test]
    fn linear_to_srgb_below_threshold() {
        let val = 0.003_f32; // below 0.003_130_8
        let srgb = linear_to_srgb(val);
        let expected = val * 12.92;
        assert!(
            (srgb - expected).abs() < 1e-7,
            "below threshold: expected {expected}, got {srgb}"
        );
    }

    #[test]
    fn linear_to_srgb_above_threshold() {
        let val = 0.5_f32;
        let srgb = linear_to_srgb(val);
        let expected = 1.055 * val.powf(1.0 / 2.4) - 0.055;
        assert!(
            (srgb - expected).abs() < 1e-7,
            "above threshold: expected {expected}, got {srgb}"
        );
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn srgb_linear_roundtrip_all_byte_values() {
        for i in 0..=255_u8 {
            let srgb_f = f32::from(i) / 255.0;
            let lin = srgb_to_linear(srgb_f);
            let back = linear_to_srgb(lin);
            let back_u8 = (back * 255.0).round() as u8;
            assert_eq!(
                i, back_u8,
                "roundtrip failed for sRGB byte {i}: linear={lin}, back_srgb={back}"
            );
        }
    }

    #[test]
    fn to_linear_monotonic() {
        // Linear values must be monotonically non-decreasing as sRGB increases.
        let mut prev = 0.0_f32;
        for i in 0..=255_u8 {
            let lin = srgb_to_linear(f32::from(i) / 255.0);
            assert!(
                lin >= prev,
                "monotonicity violated at sRGB {i}: {lin} < {prev}"
            );
            prev = lin;
        }
    }

    #[test]
    fn to_linear_values_in_unit_range() {
        // All linear values for valid sRGB must be in [0, 1].
        for i in 0..=255_u8 {
            let lin = srgb_to_linear(f32::from(i) / 255.0);
            assert!(
                (0.0..=1.0).contains(&lin),
                "linear value out of range for sRGB {i}: {lin}"
            );
        }
    }

    #[test]
    fn linear_to_srgb_clamps_negative() {
        // Negative linear input should clamp to 0.
        let result = linear_to_srgb(-0.5);
        assert!(
            result.abs() < 1e-7,
            "expected ~0 for negative input, got {result}"
        );
    }

    #[test]
    fn linear_to_srgb_clamps_above_one() {
        // Input > 1 should clamp to 1.
        let result = linear_to_srgb(1.5);
        assert!(
            (result - 1.0).abs() < 0.01,
            "expected ~1 for input > 1, got {result}"
        );
    }

    // -- Gamma roundtrip for all Nord colors -------------------------------

    #[test]
    fn linear_roundtrip_all_nord_colors() {
        let all_colors: Vec<Color> = NORD
            .polar_night
            .iter()
            .chain(NORD.snow_storm.iter())
            .chain(NORD.frost.iter())
            .chain(NORD.aurora.iter())
            .copied()
            .collect();
        for c in all_colors {
            let linear = c.to_linear();
            let back = Color::from_linear(linear);
            assert_eq!(
                c, back,
                "linear roundtrip failed for {c}: linear={linear:?}"
            );
        }
    }

    // -- from_rgb_f32 edge cases -------------------------------------------

    #[test]
    fn from_rgb_f32_exact_boundaries() {
        let c = Color::from_rgb_f32([0.0, 1.0, 0.0]);
        assert_eq!(c, Color::new(0, 255, 0));
    }

    #[test]
    fn from_rgb_f32_negative_clamps_to_zero() {
        let c = Color::from_rgb_f32([-1.0, -0.001, -100.0]);
        assert_eq!(c, Color::new(0, 0, 0));
    }

    #[test]
    fn from_rgb_f32_above_one_clamps_to_255() {
        let c = Color::from_rgb_f32([1.001, 2.0, 100.0]);
        assert_eq!(c, Color::new(255, 255, 255));
    }

    // -- Lerp edge cases ---------------------------------------------------

    #[test]
    fn lerp_negative_t_clamps_to_zero() {
        let a = Color::new(100, 100, 100);
        let b = Color::new(200, 200, 200);
        assert_eq!(a.lerp(&b, -0.5), a);
    }

    #[test]
    fn lerp_t_above_one_clamps_to_one() {
        let a = Color::new(100, 100, 100);
        let b = Color::new(200, 200, 200);
        assert_eq!(a.lerp(&b, 1.5), b);
    }

    #[test]
    fn lerp_same_color_returns_same() {
        let c = Color::new(42, 128, 200);
        assert_eq!(c.lerp(&c, 0.5), c);
    }

    #[test]
    fn lerp_non_uniform_channels() {
        let a = Color::new(0, 100, 200);
        let b = Color::new(100, 200, 50);
        let mid = a.lerp(&b, 0.5);
        assert_eq!(mid, Color::new(50, 150, 125));
    }

    #[test]
    fn lerp_quarter() {
        let a = Color::new(0, 0, 0);
        let b = Color::new(200, 200, 200);
        let quarter = a.lerp(&b, 0.25);
        assert_eq!(quarter, Color::new(50, 50, 50));
    }

    #[test]
    fn lerp_is_not_necessarily_symmetric() {
        // Due to u8 quantization, a.lerp(b, t) may differ from b.lerp(a, 1-t).
        // This test documents that behavior rather than asserting strict symmetry.
        let a = Color::new(0, 0, 0);
        let b = Color::new(255, 255, 255);
        let ab = a.lerp(&b, 0.3);
        let ba = b.lerp(&a, 0.7);
        // With 255 and 0, these happen to be the same due to exact math.
        assert_eq!(ab, ba);
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn lerp_between_nord_frost_endpoints() {
        let a = NORD.frost[0]; // #8FBCBB
        let b = NORD.frost[3]; // #5E81AC
        let mid = a.lerp(&b, 0.5);
        let expected_r = (f32::midpoint(f32::from(0x8F_u8) / 255.0, f32::from(0x5E_u8) / 255.0) * 255.0).round() as u8;
        let expected_g = (f32::midpoint(f32::from(0xBC_u8) / 255.0, f32::from(0x81_u8) / 255.0) * 255.0).round() as u8;
        let expected_b = (f32::midpoint(f32::from(0xBB_u8) / 255.0, f32::from(0xAC_u8) / 255.0) * 255.0).round() as u8;
        assert_eq!(mid, Color::new(expected_r, expected_g, expected_b));
    }

    // -- with_alpha additional tests ---------------------------------------

    #[test]
    fn with_alpha_zero() {
        let c = Color::new(128, 64, 32);
        let rgba = c.with_alpha(0.0);
        assert!((rgba[3]).abs() < f32::EPSILON);
    }

    #[test]
    fn with_alpha_one() {
        let c = Color::new(128, 64, 32);
        let rgba = c.with_alpha(1.0);
        assert!((rgba[3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn with_alpha_rgb_channels_match_to_rgb_f32() {
        let c = Color::new(0xBF, 0x61, 0x6A);
        let rgb = c.to_rgb_f32();
        let rgba = c.with_alpha(0.75);
        assert_eq!(rgba[0], rgb[0]);
        assert_eq!(rgba[1], rgb[1]);
        assert_eq!(rgba[2], rgb[2]);
    }

    // -- to_linear known reference values ----------------------------------

    #[test]
    fn to_linear_pure_red() {
        // sRGB (255, 0, 0) -> linear (1.0, 0.0, 0.0)
        let lin = Color::new(255, 0, 0).to_linear();
        assert!((lin[0] - 1.0).abs() < 1e-5);
        assert!(lin[1].abs() < 1e-7);
        assert!(lin[2].abs() < 1e-7);
    }

    #[test]
    fn to_linear_srgb_half_is_not_linear_half() {
        // sRGB 0.5 (byte ~128) should NOT map to linear 0.5
        let lin = Color::new(128, 128, 128).to_linear();
        for ch in lin {
            assert!(
                (ch - 0.5).abs() > 0.1,
                "linear should differ significantly from 0.5, got {ch}"
            );
        }
    }

    // -- Color copy/clone/eq semantics ------------------------------------

    #[test]
    fn color_copy_semantics() {
        let a = Color::new(10, 20, 30);
        let b = a; // Copy
        assert_eq!(a, b);
    }

    #[test]
    fn color_clone_equals_original() {
        let a = Color::new(10, 20, 30);
        #[allow(clippy::clone_on_copy)]
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn color_ne_different_channels() {
        let a = Color::new(10, 20, 30);
        let b = Color::new(10, 20, 31);
        assert_ne!(a, b);
    }

    // -- Serde additional tests -------------------------------------------

    #[test]
    fn serde_color_json_structure() {
        let c = Color::new(0xFF, 0x00, 0x80);
        let json = serde_json::to_string(&c).unwrap();
        // Verify the JSON contains expected field names
        assert!(json.contains("\"r\":255"), "JSON: {json}");
        assert!(json.contains("\"g\":0"), "JSON: {json}");
        assert!(json.contains("\"b\":128"), "JSON: {json}");
    }

    #[test]
    fn serde_color_from_json_object() {
        let json = r#"{"r":46,"g":52,"b":64}"#;
        let c: Color = serde_json::from_str(json).unwrap();
        assert_eq!(c, Color::new(0x2E, 0x34, 0x40));
    }

    // -- Display additional tests -----------------------------------------

    #[test]
    fn display_black() {
        let c = Color::new(0, 0, 0);
        assert_eq!(format!("{c}"), "#000000");
    }

    #[test]
    fn display_white() {
        let c = Color::new(255, 255, 255);
        assert_eq!(format!("{c}"), "#FFFFFF");
    }

    // -- FromStr tests -------------------------------------------------------

    #[test]
    fn from_str_with_hash() {
        let c: Color = "#88C0D0".parse().unwrap();
        assert_eq!(c, Color::new(0x88, 0xC0, 0xD0));
    }

    #[test]
    fn from_str_without_hash() {
        let c: Color = "2E3440".parse().unwrap();
        assert_eq!(c, Color::new(0x2E, 0x34, 0x40));
    }

    #[test]
    fn from_str_invalid_returns_error() {
        let err = "ZZZ".parse::<Color>().unwrap_err();
        assert_eq!(err, HexParseError::InvalidLength(3));
    }

    // -- NordPalette::all_colors tests ----------------------------------------

    #[test]
    fn all_colors_has_16_entries() {
        assert_eq!(NORD.all_colors().len(), 16);
    }

    #[test]
    fn all_colors_starts_with_polar_night() {
        let all = NORD.all_colors();
        for (i, &c) in NORD.polar_night.iter().enumerate() {
            assert_eq!(all[i], c, "mismatch at index {i}");
        }
    }

    #[test]
    fn all_colors_ends_with_aurora() {
        let all = NORD.all_colors();
        for (i, &c) in NORD.aurora.iter().enumerate() {
            assert_eq!(all[11 + i], c, "mismatch at aurora index {i}");
        }
    }

    #[test]
    fn all_colors_contains_every_subgroup() {
        let all = NORD.all_colors();
        let mut offset = 0;
        for &c in &NORD.polar_night {
            assert_eq!(all[offset], c);
            offset += 1;
        }
        for &c in &NORD.snow_storm {
            assert_eq!(all[offset], c);
            offset += 1;
        }
        for &c in &NORD.frost {
            assert_eq!(all[offset], c);
            offset += 1;
        }
        for &c in &NORD.aurora {
            assert_eq!(all[offset], c);
            offset += 1;
        }
        assert_eq!(offset, 16);
    }

    // -- proptest property-based tests ----------------------------------------

    mod prop {
        use super::*;
        use proptest::prelude::*;

        fn arb_color() -> impl Strategy<Value = Color> {
            (any::<u8>(), any::<u8>(), any::<u8>())
                .prop_map(|(r, g, b)| Color::new(r, g, b))
        }

        proptest! {
            #[test]
            fn hex_roundtrip(c in arb_color()) {
                let hex = c.to_hex();
                let parsed = Color::from_hex(&hex).unwrap();
                prop_assert_eq!(parsed, c);
            }

            #[test]
            fn from_str_matches_from_hex(c in arb_color()) {
                let hex = c.to_hex();
                let from_str: Color = hex.parse().unwrap();
                let from_hex = Color::from_hex(&hex).unwrap();
                prop_assert_eq!(from_str, from_hex);
            }

            #[test]
            fn display_matches_to_hex(c in arb_color()) {
                prop_assert_eq!(format!("{c}"), c.to_hex());
            }

            #[test]
            fn rgb_f32_roundtrip(c in arb_color()) {
                let floats = c.to_rgb_f32();
                let back = Color::from_rgb_f32(floats);
                prop_assert_eq!(back, c);
            }

            #[test]
            fn linear_roundtrip(c in arb_color()) {
                let linear = c.to_linear();
                let back = Color::from_linear(linear);
                prop_assert_eq!(back, c);
            }

            #[test]
            fn to_rgb_f32_in_unit_range(c in arb_color()) {
                let rgb = c.to_rgb_f32();
                for ch in rgb {
                    prop_assert!((0.0..=1.0).contains(&ch), "channel out of range: {ch}");
                }
            }

            #[test]
            fn to_linear_in_unit_range(c in arb_color()) {
                let lin = c.to_linear();
                for ch in lin {
                    prop_assert!((0.0..=1.0).contains(&ch), "channel out of range: {ch}");
                }
            }

            #[test]
            fn lerp_endpoints(a in arb_color(), b in arb_color()) {
                prop_assert_eq!(a.lerp(&b, 0.0), a);
                prop_assert_eq!(a.lerp(&b, 1.0), b);
            }

            #[test]
            fn lerp_clamps_t(a in arb_color(), b in arb_color()) {
                prop_assert_eq!(a.lerp(&b, -100.0), a);
                prop_assert_eq!(a.lerp(&b, 100.0), b);
            }

            #[test]
            fn with_alpha_preserves_rgb(c in arb_color(), alpha in 0.0_f32..=1.0) {
                let rgb = c.to_rgb_f32();
                let rgba = c.with_alpha(alpha);
                prop_assert!((rgba[0] - rgb[0]).abs() < f32::EPSILON);
                prop_assert!((rgba[1] - rgb[1]).abs() < f32::EPSILON);
                prop_assert!((rgba[2] - rgb[2]).abs() < f32::EPSILON);
            }

            #[test]
            fn with_alpha_clamps_finite(c in arb_color(), alpha in -1e6_f32..=1e6) {
                let rgba = c.with_alpha(alpha);
                prop_assert!((0.0..=1.0).contains(&rgba[3]));
            }

            #[test]
            fn from_rgb_f32_always_produces_valid_color(
                r in -1e6_f32..=1e6,
                g in -1e6_f32..=1e6,
                b in -1e6_f32..=1e6,
            ) {
                let c = Color::from_rgb_f32([r, g, b]);
                let rgb = c.to_rgb_f32();
                for ch in rgb {
                    prop_assert!((0.0..=1.0).contains(&ch));
                }
            }

            #[test]
            fn serde_json_roundtrip(c in arb_color()) {
                let json = serde_json::to_string(&c).unwrap();
                let back: Color = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(back, c);
            }

            #[test]
            fn hex_without_hash_roundtrip(c in arb_color()) {
                let hex = c.to_hex();
                let without_hash = &hex[1..];
                let parsed = Color::from_hex(without_hash).unwrap();
                prop_assert_eq!(parsed, c);
            }

            #[test]
            fn hex_parse_rejects_wrong_length(
                s in "[0-9a-fA-F]{0,5}|[0-9a-fA-F]{7,12}"
            ) {
                let result = Color::from_hex(&s);
                prop_assert!(result.is_err());
            }

            #[test]
            fn lerp_midpoint_channel_bound(
                r1 in any::<u8>(), g1 in any::<u8>(), b1 in any::<u8>(),
                r2 in any::<u8>(), g2 in any::<u8>(), b2 in any::<u8>(),
            ) {
                let a = Color::new(r1, g1, b1);
                let b = Color::new(r2, g2, b2);
                let mid = a.lerp(&b, 0.5);
                // Each channel of mid should be between min and max of a and b channels
                let check = |ca: u8, cb: u8, cm: u8| {
                    let lo = ca.min(cb);
                    let hi = ca.max(cb);
                    cm >= lo && cm <= hi
                };
                prop_assert!(check(a.r, b.r, mid.r), "r: {} not between {} and {}", mid.r, a.r, b.r);
                prop_assert!(check(a.g, b.g, mid.g), "g: {} not between {} and {}", mid.g, a.g, b.g);
                prop_assert!(check(a.b, b.b, mid.b), "b: {} not between {} and {}", mid.b, a.b, b.b);
            }
        }
    }
}

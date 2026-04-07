//! Semantic color mapping from palette colors to UI roles.
//!
//! Maps abstract roles (background, foreground, accent, etc.) to concrete
//! colors from a palette, providing a single point of configuration for
//! theming.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::palette::{Color, NordPalette, NORD};

/// Semantic color assignments for UI rendering.
///
/// Each field maps a UI role to a concrete [`Color`]. This is the primary
/// type consumers use for theming — swap the `SemanticColors` instance to
/// change the entire application theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticColors {
    /// Primary background color (darkest surface).
    pub background: Color,
    /// Primary foreground / text color.
    pub foreground: Color,
    /// Primary accent color (focus, active elements).
    pub accent: Color,
    /// Selection / highlight color.
    pub selection: Color,
    /// Error / danger indicator.
    pub error: Color,
    /// Warning indicator.
    pub warning: Color,
    /// Success indicator.
    pub success: Color,
    /// Muted / secondary text color.
    pub muted: Color,
    /// Border / separator color.
    pub border: Color,
}

impl SemanticColors {
    /// Returns the default Nord-based semantic color mapping.
    #[must_use]
    pub const fn nord() -> Self {
        Self::from_palette(&NORD)
    }

    /// Constructs semantic colors from a [`NordPalette`].
    #[must_use]
    pub const fn from_palette(palette: &NordPalette) -> Self {
        Self {
            background: palette.polar_night[0], // nord0  #2E3440
            foreground: palette.snow_storm[0],   // nord4  #D8DEE9
            accent:     palette.frost[1],        // nord8  #88C0D0
            selection:  palette.frost[2],        // nord9  #81A1C1
            error:      palette.aurora[0],       // nord11 #BF616A
            warning:    palette.aurora[2],       // nord13 #EBCB8B
            success:    palette.aurora[3],       // nord14 #A3BE8C
            muted:      palette.polar_night[3],  // nord3  #4C566A
            border:     palette.polar_night[2],  // nord2  #434C5E
        }
    }
}

impl Default for SemanticColors {
    fn default() -> Self {
        Self::nord()
    }
}

impl fmt::Display for SemanticColors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SemanticColors {{ bg: {}, fg: {}, accent: {}, sel: {}, \
             err: {}, warn: {}, ok: {}, muted: {}, border: {} }}",
            self.background,
            self.foreground,
            self.accent,
            self.selection,
            self.error,
            self.warning,
            self.success,
            self.muted,
            self.border,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nord_background_is_nord0() {
        let colors = SemanticColors::nord();
        assert_eq!(colors.background.to_hex(), "#2E3440");
    }

    #[test]
    fn nord_foreground_is_nord4() {
        let colors = SemanticColors::nord();
        assert_eq!(colors.foreground.to_hex(), "#D8DEE9");
    }

    #[test]
    fn nord_accent_is_nord8() {
        let colors = SemanticColors::nord();
        assert_eq!(colors.accent.to_hex(), "#88C0D0");
    }

    #[test]
    fn nord_selection_is_nord9() {
        let colors = SemanticColors::nord();
        assert_eq!(colors.selection.to_hex(), "#81A1C1");
    }

    #[test]
    fn nord_error_is_nord11() {
        let colors = SemanticColors::nord();
        assert_eq!(colors.error.to_hex(), "#BF616A");
    }

    #[test]
    fn nord_warning_is_nord13() {
        let colors = SemanticColors::nord();
        assert_eq!(colors.warning.to_hex(), "#EBCB8B");
    }

    #[test]
    fn nord_success_is_nord14() {
        let colors = SemanticColors::nord();
        assert_eq!(colors.success.to_hex(), "#A3BE8C");
    }

    #[test]
    fn nord_muted_is_nord3() {
        let colors = SemanticColors::nord();
        assert_eq!(colors.muted.to_hex(), "#4C566A");
    }

    #[test]
    fn nord_border_is_nord2() {
        let colors = SemanticColors::nord();
        assert_eq!(colors.border.to_hex(), "#434C5E");
    }

    #[test]
    fn default_is_nord() {
        assert_eq!(SemanticColors::default(), SemanticColors::nord());
    }

    #[test]
    fn from_palette_matches_nord() {
        assert_eq!(SemanticColors::from_palette(&NORD), SemanticColors::nord());
    }

    #[test]
    fn semantic_colors_are_serializable() {
        let colors = SemanticColors::nord();
        let json = serde_json::to_string(&colors).unwrap();
        let deserialized: SemanticColors = serde_json::from_str(&json).unwrap();
        assert_eq!(colors, deserialized);
    }

    #[test]
    fn display_contains_hex_values() {
        let colors = SemanticColors::nord();
        let display = format!("{colors}");
        assert!(display.contains("#2E3440"), "should contain background hex");
        assert!(display.contains("#88C0D0"), "should contain accent hex");
    }

    // -- Custom palette mapping -------------------------------------------

    #[test]
    fn from_custom_palette_maps_correctly() {
        let custom = NordPalette {
            polar_night: [
                Color::new(10, 10, 10),
                Color::new(20, 20, 20),
                Color::new(30, 30, 30),
                Color::new(40, 40, 40),
            ],
            snow_storm: [
                Color::new(200, 200, 200),
                Color::new(210, 210, 210),
                Color::new(220, 220, 220),
            ],
            frost: [
                Color::new(50, 100, 150),
                Color::new(60, 110, 160),
                Color::new(70, 120, 170),
                Color::new(80, 130, 180),
            ],
            aurora: [
                Color::new(200, 50, 50),
                Color::new(210, 100, 50),
                Color::new(220, 180, 80),
                Color::new(100, 180, 100),
                Color::new(150, 100, 150),
            ],
        };
        let sem = SemanticColors::from_palette(&custom);
        assert_eq!(sem.background, custom.polar_night[0]);
        assert_eq!(sem.foreground, custom.snow_storm[0]);
        assert_eq!(sem.accent, custom.frost[1]);
        assert_eq!(sem.selection, custom.frost[2]);
        assert_eq!(sem.error, custom.aurora[0]);
        assert_eq!(sem.warning, custom.aurora[2]);
        assert_eq!(sem.success, custom.aurora[3]);
        assert_eq!(sem.muted, custom.polar_night[3]);
        assert_eq!(sem.border, custom.polar_night[2]);
    }

    // -- All fields are distinct for Nord ---------------------------------

    #[test]
    fn nord_semantic_fields_are_distinct_colors() {
        let c = SemanticColors::nord();
        let all = [
            c.background, c.foreground, c.accent, c.selection,
            c.error, c.warning, c.success, c.muted, c.border,
        ];
        // Each pair should be distinct (no duplicates in the Nord mapping).
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(
                    all[i], all[j],
                    "fields at index {i} and {j} should differ"
                );
            }
        }
    }

    // -- Semantic colors to_linear produces valid GPU values ---------------

    #[test]
    fn semantic_colors_linear_values_in_range() {
        let c = SemanticColors::nord();
        let all = [
            c.background, c.foreground, c.accent, c.selection,
            c.error, c.warning, c.success, c.muted, c.border,
        ];
        for color in all {
            let lin = color.to_linear();
            for (i, ch) in lin.iter().enumerate() {
                assert!(
                    (0.0..=1.0).contains(ch),
                    "channel {i} of {color} out of range: {ch}"
                );
            }
        }
    }

    // -- Copy/Clone/Eq semantics ------------------------------------------

    #[test]
    fn semantic_colors_copy() {
        let a = SemanticColors::nord();
        let b = a; // Copy
        assert_eq!(a, b);
    }

    #[test]
    fn semantic_colors_clone() {
        let a = SemanticColors::nord();
        #[allow(clippy::clone_on_copy)]
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn semantic_colors_ne_when_field_differs() {
        let mut a = SemanticColors::nord();
        let b = SemanticColors::nord();
        a.background = Color::new(0, 0, 0);
        assert_ne!(a, b);
    }

    // -- Display format structure -----------------------------------------

    #[test]
    fn display_contains_all_semantic_labels() {
        let display = format!("{}", SemanticColors::nord());
        for label in &["bg:", "fg:", "accent:", "sel:", "err:", "warn:", "ok:", "muted:", "border:"] {
            assert!(
                display.contains(label),
                "display should contain '{label}': {display}"
            );
        }
    }

    #[test]
    fn display_contains_all_hex_values() {
        let c = SemanticColors::nord();
        let display = format!("{c}");
        let expected_hexes = [
            c.background.to_hex(),
            c.foreground.to_hex(),
            c.accent.to_hex(),
            c.selection.to_hex(),
            c.error.to_hex(),
            c.warning.to_hex(),
            c.success.to_hex(),
            c.muted.to_hex(),
            c.border.to_hex(),
        ];
        for hex in &expected_hexes {
            assert!(
                display.contains(hex.as_str()),
                "display should contain {hex}: {display}"
            );
        }
    }

    // -- Serde field names -----------------------------------------------

    #[test]
    fn serde_semantic_json_field_names() {
        let json = serde_json::to_string(&SemanticColors::nord()).unwrap();
        for field in &[
            "background", "foreground", "accent", "selection",
            "error", "warning", "success", "muted", "border",
        ] {
            assert!(
                json.contains(field),
                "JSON should contain field '{field}': {json}"
            );
        }
    }

    // -- all_colors integration -----------------------------------------------

    #[test]
    fn semantic_colors_are_subset_of_palette() {
        let all = NORD.all_colors();
        let c = SemanticColors::nord();
        let semantic = [
            c.background, c.foreground, c.accent, c.selection,
            c.error, c.warning, c.success, c.muted, c.border,
        ];
        for s in &semantic {
            assert!(
                all.contains(s),
                "semantic color {s} not found in palette"
            );
        }
    }

    // -- Display roundtrip via individual fields ------------------------------

    #[test]
    fn display_output_is_not_empty() {
        let display = format!("{}", SemanticColors::nord());
        assert!(!display.is_empty());
        assert!(display.starts_with("SemanticColors {"));
    }

    // -- Serde deserialization from known JSON ---------------------------------

    #[test]
    fn serde_deserialize_from_known_json() {
        let nord = SemanticColors::nord();
        let json = serde_json::to_string_pretty(&nord).unwrap();
        let back: SemanticColors = serde_json::from_str(&json).unwrap();
        assert_eq!(nord, back);
    }

    // -- proptest property tests for SemanticColors ----------------------------

    mod prop {
        use super::*;
        use proptest::prelude::*;

        fn arb_color() -> impl Strategy<Value = Color> {
            (any::<u8>(), any::<u8>(), any::<u8>())
                .prop_map(|(r, g, b)| Color::new(r, g, b))
        }

        fn arb_palette() -> impl Strategy<Value = NordPalette> {
            (
                proptest::array::uniform4(arb_color()),
                proptest::array::uniform3(arb_color()),
                proptest::array::uniform4(arb_color()),
                proptest::array::uniform5(arb_color()),
            )
                .prop_map(|(pn, ss, fr, au)| NordPalette {
                    polar_night: pn,
                    snow_storm: ss,
                    frost: fr,
                    aurora: au,
                })
        }

        proptest! {
            #[test]
            fn from_palette_maps_expected_slots(p in arb_palette()) {
                let s = SemanticColors::from_palette(&p);
                prop_assert_eq!(s.background, p.polar_night[0]);
                prop_assert_eq!(s.foreground, p.snow_storm[0]);
                prop_assert_eq!(s.accent, p.frost[1]);
                prop_assert_eq!(s.selection, p.frost[2]);
                prop_assert_eq!(s.error, p.aurora[0]);
                prop_assert_eq!(s.warning, p.aurora[2]);
                prop_assert_eq!(s.success, p.aurora[3]);
                prop_assert_eq!(s.muted, p.polar_night[3]);
                prop_assert_eq!(s.border, p.polar_night[2]);
            }

            #[test]
            fn serde_roundtrip(p in arb_palette()) {
                let s = SemanticColors::from_palette(&p);
                let json = serde_json::to_string(&s).unwrap();
                let back: SemanticColors = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(s, back);
            }

            #[test]
            fn display_contains_all_field_hexes(p in arb_palette()) {
                let s = SemanticColors::from_palette(&p);
                let display = format!("{s}");
                prop_assert!(display.contains(&s.background.to_hex()));
                prop_assert!(display.contains(&s.foreground.to_hex()));
                prop_assert!(display.contains(&s.accent.to_hex()));
                prop_assert!(display.contains(&s.error.to_hex()));
            }

            #[test]
            fn palette_serde_roundtrip(p in arb_palette()) {
                let json = serde_json::to_string(&p).unwrap();
                let back: NordPalette = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(p, back);
            }
        }
    }
}

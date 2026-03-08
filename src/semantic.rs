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
}

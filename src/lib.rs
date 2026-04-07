//! Irodori (彩り) — theme and color system.
//!
//! Provides the Nord color palette, sRGB/linear conversion, and semantic
//! color mapping for consistent theming across pleme-io applications.
//!
//! # Quick Start
//!
//! ```
//! use irodori::{Color, SemanticColors, NORD};
//!
//! // Use the full Nord palette directly
//! let bg = NORD.polar_night[0];
//! assert_eq!(bg.to_hex(), "#2E3440");
//!
//! // Or use semantic mappings for UI rendering
//! let theme = SemanticColors::nord();
//! let linear_bg = theme.background.to_linear();
//! ```

pub mod palette;
pub mod semantic;

pub use palette::{Color, HexParseError, NordPalette, NORD};
pub use semantic::{SemanticColors, SemanticColorsBuilder};

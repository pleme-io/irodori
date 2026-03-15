# Irodori (彩り) — Theme & Color System

## Build & Test

```bash
cargo build          # compile
cargo test           # 96 unit tests + 1 doc-test
```

## Architecture

Shared color and theming library providing:
- Nord color palette as compile-time constants (u8 sRGB channels)
- sRGB / linear RGB conversion with correct transfer functions
- Hex color parsing and formatting with proper error type
- Float conversion, interpolation (lerp), and alpha compositing
- Semantic color mapping from palette to UI roles

### Module Map

| Path | Purpose |
|------|---------|
| `src/lib.rs` | Re-exports Color, HexParseError, NordPalette, SemanticColors, NORD |
| `src/palette.rs` | `Color` (u8 sRGB), `NordPalette`, `NORD` const, `HexParseError` (75 tests) |
| `src/semantic.rs` | `SemanticColors` — UI role to color mapping via `from_palette()` (21 tests) |

### Key Types

- **`Color`** — u8 sRGB color with `to_linear()`, `from_linear()`, `to_hex()`, `from_hex()`, `to_rgb_f32()`, `from_rgb_f32()`, `lerp()`, `with_alpha()`
- **`HexParseError`** — `InvalidLength(usize)` or `InvalidChar`
- **`NordPalette`** — four const arrays: polar_night[4], snow_storm[3], frost[4], aurora[5]
- **`NORD`** — canonical Nord palette constant
- **`SemanticColors`** — 9-field struct mapping UI roles to Colors, constructable via `nord()` or `from_palette()`

### Usage Pattern

```rust
use irodori::{Color, SemanticColors, NORD};

// Direct palette access
let frost_blue = NORD.frost[1]; // #88C0D0

// Semantic theming
let theme = SemanticColors::nord();
let bg_linear = theme.background.to_linear(); // for GPU shaders

// Custom colors
let custom = Color::from_hex("#FF6600").unwrap();
let mid = NORD.frost[0].lerp(&NORD.frost[3], 0.5);
```

## Consumers

- **tobira** — app launcher theme colors
- **ayatsuri** — window manager border/highlight colors
- **hikyaku** — email client UI theme

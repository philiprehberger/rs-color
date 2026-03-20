# rs-color

[![CI](https://github.com/philiprehberger/rs-color/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-color/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-color.svg)](https://crates.io/crates/philiprehberger-color)
[![License](https://img.shields.io/github/license/philiprehberger/rs-color)](LICENSE)

Color manipulation library — parsing, conversion, blending, contrast checking, and ANSI terminal output

## Installation

```toml
[dependencies]
philiprehberger-color = "0.1"
```

## Usage

```rust
use philiprehberger_color::Color;

// Create colors
let red = Color::rgb(255, 0, 0);
let blue = Color::from_hex("#0000ff").unwrap();
let coral = Color::named("coral").unwrap();

// Convert between color spaces
let (h, s, l) = red.to_hsl();
let hex = blue.to_hex(); // "#0000ff"

// Manipulate
let lighter = red.lighten(0.3);
let muted = red.desaturate(0.5);
let mixed = Color::mix(red, blue, 0.5); // purple

// Check contrast (WCAG 2.1)
let ratio = Color::rgb(0, 0, 0).contrast_ratio(Color::rgb(255, 255, 255));
assert!(Color::rgb(0, 0, 0).meets_wcag_aa(Color::rgb(255, 255, 255)));

// Terminal output
println!("{}", red.ansi_paint("This is red text"));
```

## API

| Function / Type | Description |
|----------------|-------------|
| `Color::rgb(r, g, b)` | Create from RGB values |
| `Color::from_hex(hex)` | Parse hex color string |
| `Color::from_hsl(h, s, l)` | Create from HSL values |
| `Color::from_hsv(h, s, v)` | Create from HSV values |
| `Color::named(name)` | Lookup CSS named color |
| `Color::mix(a, b, ratio)` | Blend two colors |
| `.lighten(amount)` | Increase lightness |
| `.darken(amount)` | Decrease lightness |
| `.saturate(amount)` | Increase saturation |
| `.desaturate(amount)` | Decrease saturation |
| `.invert()` | Invert color |
| `.grayscale()` | Convert to grayscale |
| `.contrast_ratio(other)` | WCAG contrast ratio |
| `.meets_wcag_aa(other)` | Check WCAG AA compliance |
| `.to_hex()` | Output as hex string |
| `.to_ansi_fg()` | ANSI foreground escape |
| `.ansi_paint(text)` | Wrap text with color |

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## License

MIT

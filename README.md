# rs-color

[![CI](https://github.com/philiprehberger/rs-color/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-color/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-color.svg)](https://crates.io/crates/philiprehberger-color)
[![GitHub release](https://img.shields.io/github/v/release/philiprehberger/rs-color)](https://github.com/philiprehberger/rs-color/releases)
[![Last updated](https://img.shields.io/github/last-commit/philiprehberger/rs-color)](https://github.com/philiprehberger/rs-color/commits/main)
[![License](https://img.shields.io/github/license/philiprehberger/rs-color)](LICENSE)
[![Bug Reports](https://img.shields.io/github/issues/philiprehberger/rs-color/bug)](https://github.com/philiprehberger/rs-color/issues?q=is%3Aissue+is%3Aopen+label%3Abug)
[![Feature Requests](https://img.shields.io/github/issues/philiprehberger/rs-color/enhancement)](https://github.com/philiprehberger/rs-color/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement)
[![Sponsor](https://img.shields.io/badge/sponsor-GitHub%20Sponsors-ec6cb9)](https://github.com/sponsors/philiprehberger)

Color manipulation library — parsing, conversion, blending, contrast checking, and ANSI terminal output

## Installation

```toml
[dependencies]
philiprehberger-color = "0.2.1"
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

// Display and FromStr
println!("{}", red); // "#ff0000"
let parsed: Color = "#00ff00".parse().unwrap();

// Color harmony
let comp = red.complementary(); // cyan
let [t1, t2] = red.triadic();  // 120° and 240° rotations

// Gradient
let steps = red.gradient(&blue, 5); // 5 colors from red to blue

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
| `.complementary()` | 180° hue rotation |
| `.triadic()` | Two colors at 120° intervals |
| `.gradient(other, steps)` | Interpolate between two colors |
| `Display` / `FromStr` | Print as hex, parse from hex |
| `Default` | White (255, 255, 255) |

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## Support

If you find this package useful, consider giving it a star on GitHub — it helps motivate continued maintenance and development.

[![LinkedIn](https://img.shields.io/badge/Philip%20Rehberger-LinkedIn-0A66C2?logo=linkedin)](https://www.linkedin.com/in/philiprehberger)
[![More packages](https://img.shields.io/badge/more-open%20source%20packages-blue)](https://philiprehberger.com/open-source-packages)

## License

[MIT](LICENSE)

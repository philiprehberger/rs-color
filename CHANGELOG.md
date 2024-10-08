# Changelog

## 0.2.0 (2026-03-20)

- Add Display trait implementation (outputs hex format)
- Add FromStr trait implementation (parses hex color strings)
- Add complementary() method for complementary color calculation
- Add triadic() method for triadic color harmony
- Add gradient() method for color interpolation
- Add Default trait implementation (white)
- Add #[must_use] attributes on transformation methods

## 0.1.0 (2026-03-19)

- Initial release
- Color struct with RGB, HSL, HSV constructors
- Parse from hex, RGB, HSL strings
- Convert between RGB, HSL, HSV color spaces
- Color operations: lighten, darken, saturate, desaturate, invert, grayscale
- Color blending/mixing with configurable ratio
- WCAG 2.1 contrast ratio calculation with AA/AAA compliance checking
- ANSI terminal output (truecolor escape sequences)
- 148 named CSS colors

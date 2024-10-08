//! Color manipulation library for parsing, conversion, blending, contrast checking,
//! and ANSI terminal output.
//!
//! # Examples
//!
//! ```
//! use philiprehberger_color::Color;
//!
//! let red = Color::rgb(255, 0, 0);
//! let blue = Color::from_hex("#0000ff").unwrap();
//!
//! let mixed = Color::mix(red, blue, 0.5);
//! let lighter = red.lighten(0.3);
//!
//! assert!(Color::rgb(0, 0, 0).meets_wcag_aa(Color::rgb(255, 255, 255)));
//! ```

use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// Error type for color parsing operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorError {
    /// The input string is not a valid hex color.
    InvalidHex(String),
    /// The input string format is not recognized.
    InvalidFormat(String),
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorError::InvalidHex(s) => write!(f, "invalid hex color: {s}"),
            ColorError::InvalidFormat(s) => write!(f, "invalid color format: {s}"),
        }
    }
}

impl std::error::Error for ColorError {}

/// A color represented internally as RGB values.
///
/// Provides constructors from RGB, hex, HSL, HSV, and named CSS colors,
/// plus conversion, manipulation, blending, contrast checking, and ANSI output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.hash(state);
        self.g.hash(state);
        self.b.hash(state);
    }
}

impl Color {
    /// Create a color from RGB component values.
    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }

    /// Parse a color from a hex string.
    ///
    /// Accepts formats: `"#ff6600"`, `"#f60"`, `"ff6600"`, `"f60"`.
    pub fn from_hex(hex: &str) -> Result<Color, ColorError> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        let expanded = match hex.len() {
            3 => {
                let chars: Vec<char> = hex.chars().collect();
                let mut s = String::with_capacity(6);
                for c in &chars {
                    if !c.is_ascii_hexdigit() {
                        return Err(ColorError::InvalidHex(hex.to_string()));
                    }
                    s.push(*c);
                    s.push(*c);
                }
                s
            }
            6 => {
                for c in hex.chars() {
                    if !c.is_ascii_hexdigit() {
                        return Err(ColorError::InvalidHex(hex.to_string()));
                    }
                }
                hex.to_string()
            }
            _ => return Err(ColorError::InvalidHex(hex.to_string())),
        };

        let r = u8::from_str_radix(&expanded[0..2], 16)
            .map_err(|_| ColorError::InvalidHex(hex.to_string()))?;
        let g = u8::from_str_radix(&expanded[2..4], 16)
            .map_err(|_| ColorError::InvalidHex(hex.to_string()))?;
        let b = u8::from_str_radix(&expanded[4..6], 16)
            .map_err(|_| ColorError::InvalidHex(hex.to_string()))?;

        Ok(Color { r, g, b })
    }

    /// Create a color from HSL values.
    ///
    /// - `h`: hue in degrees (0-360)
    /// - `s`: saturation (0.0-1.0)
    /// - `l`: lightness (0.0-1.0)
    pub fn from_hsl(h: f64, s: f64, l: f64) -> Color {
        let (r, g, b) = hsl_to_rgb(h, s, l);
        Color { r, g, b }
    }

    /// Create a color from HSV values.
    ///
    /// - `h`: hue in degrees (0-360)
    /// - `s`: saturation (0.0-1.0)
    /// - `v`: value/brightness (0.0-1.0)
    pub fn from_hsv(h: f64, s: f64, v: f64) -> Color {
        let (r, g, b) = hsv_to_rgb(h, s, v);
        Color { r, g, b }
    }

    /// Look up a CSS named color by name (case-insensitive).
    ///
    /// Returns `None` if the name is not recognized.
    pub fn named(name: &str) -> Option<Color> {
        let lower = name.to_lowercase();
        NAMED_COLORS
            .iter()
            .find(|(n, _, _, _)| *n == lower)
            .map(|(_, r, g, b)| Color::rgb(*r, *g, *b))
    }

    /// Returns the red component.
    pub fn r(&self) -> u8 {
        self.r
    }

    /// Returns the green component.
    pub fn g(&self) -> u8 {
        self.g
    }

    /// Returns the blue component.
    pub fn b(&self) -> u8 {
        self.b
    }

    /// Convert to HSL representation.
    ///
    /// Returns `(h, s, l)` where h is 0-360, s and l are 0.0-1.0.
    pub fn to_hsl(&self) -> (f64, f64, f64) {
        rgb_to_hsl(self.r, self.g, self.b)
    }

    /// Convert to HSV representation.
    ///
    /// Returns `(h, s, v)` where h is 0-360, s and v are 0.0-1.0.
    pub fn to_hsv(&self) -> (f64, f64, f64) {
        rgb_to_hsv(self.r, self.g, self.b)
    }

    /// Convert to a hex string like `"#ff6600"`.
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Convert to an RGB string like `"rgb(255, 102, 0)"`.
    pub fn to_rgb_string(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    /// Convert to an HSL string like `"hsl(24, 100%, 50%)"`.
    pub fn to_hsl_string(&self) -> String {
        let (h, s, l) = self.to_hsl();
        format!(
            "hsl({}, {}%, {}%)",
            h.round() as i32,
            (s * 100.0).round() as i32,
            (l * 100.0).round() as i32
        )
    }

    /// Increase lightness in HSL space.
    ///
    /// `amount` is 0.0-1.0, added to the current lightness (clamped to 1.0).
    #[must_use]
    pub fn lighten(&self, amount: f64) -> Color {
        let (h, s, l) = self.to_hsl();
        let l = (l + amount).clamp(0.0, 1.0);
        Color::from_hsl(h, s, l)
    }

    /// Decrease lightness in HSL space.
    ///
    /// `amount` is 0.0-1.0, subtracted from the current lightness (clamped to 0.0).
    #[must_use]
    pub fn darken(&self, amount: f64) -> Color {
        let (h, s, l) = self.to_hsl();
        let l = (l - amount).clamp(0.0, 1.0);
        Color::from_hsl(h, s, l)
    }

    /// Increase saturation in HSL space.
    ///
    /// `amount` is 0.0-1.0, added to the current saturation (clamped to 1.0).
    #[must_use]
    pub fn saturate(&self, amount: f64) -> Color {
        let (h, s, l) = self.to_hsl();
        let s = (s + amount).clamp(0.0, 1.0);
        Color::from_hsl(h, s, l)
    }

    /// Decrease saturation in HSL space.
    ///
    /// `amount` is 0.0-1.0, subtracted from the current saturation (clamped to 0.0).
    #[must_use]
    pub fn desaturate(&self, amount: f64) -> Color {
        let (h, s, l) = self.to_hsl();
        let s = (s - amount).clamp(0.0, 1.0);
        Color::from_hsl(h, s, l)
    }

    /// Invert the color (255 - each component).
    #[must_use]
    pub fn invert(&self) -> Color {
        Color {
            r: 255 - self.r,
            g: 255 - self.g,
            b: 255 - self.b,
        }
    }

    /// Convert to grayscale using the luminance formula.
    ///
    /// Uses ITU-R BT.709 coefficients: 0.2126*R + 0.7152*G + 0.0722*B.
    #[must_use]
    pub fn grayscale(&self) -> Color {
        let gray = (0.2126 * self.r as f64 + 0.7152 * self.g as f64 + 0.0722 * self.b as f64)
            .round() as u8;
        Color {
            r: gray,
            g: gray,
            b: gray,
        }
    }

    /// Rotate the hue by the given number of degrees in HSL space.
    #[must_use]
    pub fn rotate_hue(&self, degrees: f64) -> Color {
        let (h, s, l) = self.to_hsl();
        let h = ((h + degrees) % 360.0 + 360.0) % 360.0;
        Color::from_hsl(h, s, l)
    }

    /// Linearly interpolate between two colors in RGB space.
    ///
    /// `ratio` of 0.0 returns `a`, 1.0 returns `b`.
    #[must_use]
    pub fn mix(a: Color, b: Color, ratio: f64) -> Color {
        let ratio = ratio.clamp(0.0, 1.0);
        let r = (a.r as f64 * (1.0 - ratio) + b.r as f64 * ratio).round() as u8;
        let g = (a.g as f64 * (1.0 - ratio) + b.g as f64 * ratio).round() as u8;
        let bl = (a.b as f64 * (1.0 - ratio) + b.b as f64 * ratio).round() as u8;
        Color { r, g, b: bl }
    }

    /// Return the complementary color (180° hue rotation).
    #[must_use]
    pub fn complementary(&self) -> Color {
        self.rotate_hue(180.0)
    }

    /// Return the two triadic harmony colors (+120° and +240° hue rotation).
    #[must_use]
    pub fn triadic(&self) -> [Color; 2] {
        [self.rotate_hue(120.0), self.rotate_hue(240.0)]
    }

    /// Linearly interpolate between this color and another, producing `steps` colors.
    ///
    /// Returns a `Vec` of evenly spaced colors from `self` to `other` (inclusive).
    #[must_use]
    pub fn gradient(&self, other: &Color, steps: usize) -> Vec<Color> {
        if steps == 0 {
            return Vec::new();
        }
        if steps == 1 {
            return vec![*self];
        }
        let mut colors = Vec::with_capacity(steps);
        for i in 0..steps {
            let t = i as f64 / (steps - 1) as f64;
            let r = (self.r as f64 * (1.0 - t) + other.r as f64 * t).round() as u8;
            let g = (self.g as f64 * (1.0 - t) + other.g as f64 * t).round() as u8;
            let b = (self.b as f64 * (1.0 - t) + other.b as f64 * t).round() as u8;
            colors.push(Color { r, g, b });
        }
        colors
    }

    /// Calculate the relative luminance per WCAG 2.1.
    ///
    /// Returns a value between 0.0 (black) and 1.0 (white).
    pub fn luminance(&self) -> f64 {
        let r = srgb_to_linear(self.r as f64 / 255.0);
        let g = srgb_to_linear(self.g as f64 / 255.0);
        let b = srgb_to_linear(self.b as f64 / 255.0);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Calculate the WCAG 2.1 contrast ratio between this color and another.
    ///
    /// Returns a value between 1.0 and 21.0.
    pub fn contrast_ratio(&self, other: Color) -> f64 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Check if the contrast ratio meets WCAG AA for normal text (>= 4.5).
    pub fn meets_wcag_aa(&self, other: Color) -> bool {
        self.contrast_ratio(other) >= 4.5
    }

    /// Check if the contrast ratio meets WCAG AAA for normal text (>= 7.0).
    pub fn meets_wcag_aaa(&self, other: Color) -> bool {
        self.contrast_ratio(other) >= 7.0
    }

    /// Check if the contrast ratio meets WCAG AA for large text (>= 3.0).
    pub fn meets_wcag_aa_large(&self, other: Color) -> bool {
        self.contrast_ratio(other) >= 3.0
    }

    /// Generate an ANSI truecolor foreground escape sequence.
    pub fn to_ansi_fg(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Generate an ANSI truecolor background escape sequence.
    pub fn to_ansi_bg(&self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Wrap text with the foreground color and a reset sequence.
    pub fn ansi_paint(&self, text: &str) -> String {
        format!("{}{}\x1b[0m", self.to_ansi_fg(), text)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for Color {
    type Err = ColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Color::from_hex(s)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 255,
            g: 255,
            b: 255,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal conversion helpers
// ---------------------------------------------------------------------------

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < f64::EPSILON {
        let mut h = (g - b) / d;
        if g < b {
            h += 6.0;
        }
        h
    } else if (max - g).abs() < f64::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    (h * 60.0, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s.abs() < f64::EPSILON {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h = h / 360.0;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;

    let v = max;

    if max.abs() < f64::EPSILON {
        return (0.0, 0.0, v);
    }

    let s = d / max;

    if d.abs() < f64::EPSILON {
        return (0.0, s, v);
    }

    let h = if (max - r).abs() < f64::EPSILON {
        let mut h = (g - b) / d;
        if g < b {
            h += 6.0;
        }
        h
    } else if (max - g).abs() < f64::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    (h * 60.0, s, v)
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    if s.abs() < f64::EPSILON {
        let val = (v * 255.0).round() as u8;
        return (val, val, val);
    }

    let h = ((h % 360.0) + 360.0) % 360.0;
    let h = h / 60.0;
    let i = h.floor() as u32;
    let f = h - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

// ---------------------------------------------------------------------------
// Named CSS colors (148 entries)
// ---------------------------------------------------------------------------

const NAMED_COLORS: &[(&str, u8, u8, u8)] = &[
    ("aliceblue", 240, 248, 255),
    ("antiquewhite", 250, 235, 215),
    ("aqua", 0, 255, 255),
    ("aquamarine", 127, 255, 212),
    ("azure", 240, 255, 255),
    ("beige", 245, 245, 220),
    ("bisque", 255, 228, 196),
    ("black", 0, 0, 0),
    ("blanchedalmond", 255, 235, 205),
    ("blue", 0, 0, 255),
    ("blueviolet", 138, 43, 226),
    ("brown", 165, 42, 42),
    ("burlywood", 222, 184, 135),
    ("cadetblue", 95, 158, 160),
    ("chartreuse", 127, 255, 0),
    ("chocolate", 210, 105, 30),
    ("coral", 255, 127, 80),
    ("cornflowerblue", 100, 149, 237),
    ("cornsilk", 255, 248, 220),
    ("crimson", 220, 20, 60),
    ("cyan", 0, 255, 255),
    ("darkblue", 0, 0, 139),
    ("darkcyan", 0, 139, 139),
    ("darkgoldenrod", 184, 134, 11),
    ("darkgray", 169, 169, 169),
    ("darkgreen", 0, 100, 0),
    ("darkgrey", 169, 169, 169),
    ("darkkhaki", 189, 183, 107),
    ("darkmagenta", 139, 0, 139),
    ("darkolivegreen", 85, 107, 47),
    ("darkorange", 255, 140, 0),
    ("darkorchid", 153, 50, 204),
    ("darkred", 139, 0, 0),
    ("darksalmon", 233, 150, 122),
    ("darkseagreen", 143, 188, 143),
    ("darkslateblue", 72, 61, 139),
    ("darkslategray", 47, 79, 79),
    ("darkslategrey", 47, 79, 79),
    ("darkturquoise", 0, 206, 209),
    ("darkviolet", 148, 0, 211),
    ("deeppink", 255, 20, 147),
    ("deepskyblue", 0, 191, 255),
    ("dimgray", 105, 105, 105),
    ("dimgrey", 105, 105, 105),
    ("dodgerblue", 30, 144, 255),
    ("firebrick", 178, 34, 34),
    ("floralwhite", 255, 250, 240),
    ("forestgreen", 34, 139, 34),
    ("fuchsia", 255, 0, 255),
    ("gainsboro", 220, 220, 220),
    ("ghostwhite", 248, 248, 255),
    ("gold", 255, 215, 0),
    ("goldenrod", 218, 165, 32),
    ("gray", 128, 128, 128),
    ("green", 0, 128, 0),
    ("greenyellow", 173, 255, 47),
    ("grey", 128, 128, 128),
    ("honeydew", 240, 255, 240),
    ("hotpink", 255, 105, 180),
    ("indianred", 205, 92, 92),
    ("indigo", 75, 0, 130),
    ("ivory", 255, 255, 240),
    ("khaki", 240, 230, 140),
    ("lavender", 230, 230, 250),
    ("lavenderblush", 255, 240, 245),
    ("lawngreen", 124, 252, 0),
    ("lemonchiffon", 255, 250, 205),
    ("lightblue", 173, 216, 230),
    ("lightcoral", 240, 128, 128),
    ("lightcyan", 224, 255, 255),
    ("lightgoldenrodyellow", 250, 250, 210),
    ("lightgray", 211, 211, 211),
    ("lightgreen", 144, 238, 144),
    ("lightgrey", 211, 211, 211),
    ("lightpink", 255, 182, 193),
    ("lightsalmon", 255, 160, 122),
    ("lightseagreen", 32, 178, 170),
    ("lightskyblue", 135, 206, 250),
    ("lightslategray", 119, 136, 153),
    ("lightslategrey", 119, 136, 153),
    ("lightsteelblue", 176, 196, 222),
    ("lightyellow", 255, 255, 224),
    ("lime", 0, 255, 0),
    ("limegreen", 50, 205, 50),
    ("linen", 250, 240, 230),
    ("magenta", 255, 0, 255),
    ("maroon", 128, 0, 0),
    ("mediumaquamarine", 102, 205, 170),
    ("mediumblue", 0, 0, 205),
    ("mediumorchid", 186, 85, 211),
    ("mediumpurple", 147, 112, 219),
    ("mediumseagreen", 60, 179, 113),
    ("mediumslateblue", 123, 104, 238),
    ("mediumspringgreen", 0, 250, 154),
    ("mediumturquoise", 72, 209, 204),
    ("mediumvioletred", 199, 21, 133),
    ("midnightblue", 25, 25, 112),
    ("mintcream", 245, 255, 250),
    ("mistyrose", 255, 228, 225),
    ("moccasin", 255, 228, 181),
    ("navajowhite", 255, 222, 173),
    ("navy", 0, 0, 128),
    ("oldlace", 253, 245, 230),
    ("olive", 128, 128, 0),
    ("olivedrab", 107, 142, 35),
    ("orange", 255, 165, 0),
    ("orangered", 255, 69, 0),
    ("orchid", 218, 112, 214),
    ("palegoldenrod", 238, 232, 170),
    ("palegreen", 152, 251, 152),
    ("paleturquoise", 175, 238, 238),
    ("palevioletred", 219, 112, 147),
    ("papayawhip", 255, 239, 213),
    ("peachpuff", 255, 218, 185),
    ("peru", 205, 133, 63),
    ("pink", 255, 192, 203),
    ("plum", 221, 160, 221),
    ("powderblue", 176, 224, 230),
    ("purple", 128, 0, 128),
    ("rebeccapurple", 102, 51, 153),
    ("red", 255, 0, 0),
    ("rosybrown", 188, 143, 143),
    ("royalblue", 65, 105, 225),
    ("saddlebrown", 139, 69, 19),
    ("salmon", 250, 128, 114),
    ("sandybrown", 244, 164, 96),
    ("seagreen", 46, 139, 87),
    ("seashell", 255, 245, 238),
    ("sienna", 160, 82, 45),
    ("silver", 192, 192, 192),
    ("skyblue", 135, 206, 235),
    ("slateblue", 106, 90, 205),
    ("slategray", 112, 128, 144),
    ("slategrey", 112, 128, 144),
    ("snow", 255, 250, 250),
    ("springgreen", 0, 255, 127),
    ("steelblue", 70, 130, 180),
    ("tan", 210, 180, 140),
    ("teal", 0, 128, 128),
    ("thistle", 216, 191, 216),
    ("tomato", 255, 99, 71),
    ("turquoise", 64, 224, 208),
    ("violet", 238, 130, 238),
    ("wheat", 245, 222, 179),
    ("white", 255, 255, 255),
    ("whitesmoke", 245, 245, 245),
    ("yellow", 255, 255, 0),
    ("yellowgreen", 154, 205, 50),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_parsing_full() {
        let c = Color::from_hex("#ff6600").unwrap();
        assert_eq!(c, Color::rgb(255, 102, 0));
    }

    #[test]
    fn test_hex_parsing_short() {
        let c = Color::from_hex("#f60").unwrap();
        assert_eq!(c, Color::rgb(255, 102, 0));
    }

    #[test]
    fn test_hex_parsing_no_hash() {
        let c = Color::from_hex("ff6600").unwrap();
        assert_eq!(c, Color::rgb(255, 102, 0));
    }

    #[test]
    fn test_hex_parsing_short_no_hash() {
        let c = Color::from_hex("f60").unwrap();
        assert_eq!(c, Color::rgb(255, 102, 0));
    }

    #[test]
    fn test_hex_parsing_invalid() {
        assert!(Color::from_hex("#xyz").is_err());
        assert!(Color::from_hex("#12345").is_err());
        assert!(Color::from_hex("").is_err());
        assert!(Color::from_hex("#gg0000").is_err());
    }

    #[test]
    fn test_rgb_to_hsl_roundtrip() {
        let original = Color::rgb(200, 100, 50);
        let (h, s, l) = original.to_hsl();
        let converted = Color::from_hsl(h, s, l);
        assert_eq!(original, converted);
    }

    #[test]
    fn test_rgb_to_hsv_roundtrip() {
        let original = Color::rgb(200, 100, 50);
        let (h, s, v) = original.to_hsv();
        let converted = Color::from_hsv(h, s, v);
        assert_eq!(original, converted);
    }

    #[test]
    fn test_hsl_pure_red() {
        let (h, s, l) = Color::rgb(255, 0, 0).to_hsl();
        assert!((h - 0.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_lighten() {
        let black = Color::rgb(0, 0, 0);
        let lightened = black.lighten(0.5);
        // Lightened black with l=0.5 should be roughly gray
        assert!((lightened.r() as i16 - 128).abs() <= 1);
        assert!((lightened.g() as i16 - 128).abs() <= 1);
        assert!((lightened.b() as i16 - 128).abs() <= 1);
    }

    #[test]
    fn test_darken() {
        let white = Color::rgb(255, 255, 255);
        let darkened = white.darken(0.5);
        assert!((darkened.r() as i16 - 128).abs() <= 1);
    }

    #[test]
    fn test_invert() {
        assert_eq!(
            Color::rgb(255, 255, 255).invert(),
            Color::rgb(0, 0, 0)
        );
        assert_eq!(
            Color::rgb(0, 0, 0).invert(),
            Color::rgb(255, 255, 255)
        );
        assert_eq!(
            Color::rgb(255, 0, 128).invert(),
            Color::rgb(0, 255, 127)
        );
    }

    #[test]
    fn test_grayscale() {
        let white = Color::rgb(255, 255, 255).grayscale();
        assert_eq!(white, Color::rgb(255, 255, 255));

        let black = Color::rgb(0, 0, 0).grayscale();
        assert_eq!(black, Color::rgb(0, 0, 0));

        let red = Color::rgb(255, 0, 0).grayscale();
        // 0.2126 * 255 = 54.213 => 54
        assert_eq!(red.r(), 54);
        assert_eq!(red.g(), 54);
        assert_eq!(red.b(), 54);
    }

    #[test]
    fn test_mix() {
        let mixed = Color::mix(Color::rgb(0, 0, 0), Color::rgb(255, 255, 255), 0.5);
        assert!((mixed.r() as i16 - 128).abs() <= 1);
        assert!((mixed.g() as i16 - 128).abs() <= 1);
        assert!((mixed.b() as i16 - 128).abs() <= 1);
    }

    #[test]
    fn test_mix_boundaries() {
        let a = Color::rgb(255, 0, 0);
        let b = Color::rgb(0, 0, 255);
        assert_eq!(Color::mix(a, b, 0.0), a);
        assert_eq!(Color::mix(a, b, 1.0), b);
    }

    #[test]
    fn test_contrast_ratio_black_white() {
        let ratio = Color::rgb(0, 0, 0).contrast_ratio(Color::rgb(255, 255, 255));
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn test_contrast_ratio_same_color() {
        let ratio = Color::rgb(128, 128, 128).contrast_ratio(Color::rgb(128, 128, 128));
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_wcag_compliance() {
        let black = Color::rgb(0, 0, 0);
        let white = Color::rgb(255, 255, 255);
        assert!(black.meets_wcag_aa(white));
        assert!(black.meets_wcag_aaa(white));
        assert!(black.meets_wcag_aa_large(white));
    }

    #[test]
    fn test_named_colors() {
        let tomato = Color::named("tomato").unwrap();
        assert_eq!(tomato, Color::rgb(255, 99, 71));

        let coral = Color::named("coral").unwrap();
        assert_eq!(coral, Color::rgb(255, 127, 80));

        assert!(Color::named("nonexistent").is_none());
    }

    #[test]
    fn test_named_colors_case_insensitive() {
        let blue = Color::named("Blue").unwrap();
        assert_eq!(blue, Color::rgb(0, 0, 255));
    }

    #[test]
    fn test_ansi_fg() {
        let c = Color::rgb(255, 0, 128);
        assert_eq!(c.to_ansi_fg(), "\x1b[38;2;255;0;128m");
    }

    #[test]
    fn test_ansi_bg() {
        let c = Color::rgb(255, 0, 128);
        assert_eq!(c.to_ansi_bg(), "\x1b[48;2;255;0;128m");
    }

    #[test]
    fn test_ansi_paint() {
        let c = Color::rgb(255, 0, 0);
        let painted = c.ansi_paint("hello");
        assert_eq!(painted, "\x1b[38;2;255;0;0mhello\x1b[0m");
    }

    #[test]
    fn test_display() {
        let c = Color::rgb(255, 102, 0);
        assert_eq!(format!("{c}"), "#ff6600");
    }

    #[test]
    fn test_from_str_roundtrip() {
        let original = Color::rgb(255, 102, 0);
        let hex = original.to_hex();
        let parsed: Color = hex.parse().unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_to_hex() {
        assert_eq!(Color::rgb(0, 0, 0).to_hex(), "#000000");
        assert_eq!(Color::rgb(255, 255, 255).to_hex(), "#ffffff");
        assert_eq!(Color::rgb(255, 102, 0).to_hex(), "#ff6600");
    }

    #[test]
    fn test_to_rgb_string() {
        assert_eq!(
            Color::rgb(255, 102, 0).to_rgb_string(),
            "rgb(255, 102, 0)"
        );
    }

    #[test]
    fn test_to_hsl_string() {
        let c = Color::rgb(255, 0, 0);
        assert_eq!(c.to_hsl_string(), "hsl(0, 100%, 50%)");
    }

    #[test]
    fn test_rotate_hue() {
        let red = Color::rgb(255, 0, 0);
        let rotated = red.rotate_hue(120.0);
        // Red rotated 120 degrees should be approximately green
        assert!(rotated.g() > 200);
        assert!(rotated.r() < 10);
    }

    #[test]
    fn test_saturate_desaturate() {
        let c = Color::from_hsl(0.0, 0.5, 0.5);
        let saturated = c.saturate(0.3);
        let (_, s, _) = saturated.to_hsl();
        assert!((s - 0.8).abs() < 0.02);

        let desaturated = c.desaturate(0.3);
        let (_, s, _) = desaturated.to_hsl();
        assert!((s - 0.2).abs() < 0.02);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Color::rgb(255, 0, 0));
        set.insert(Color::rgb(255, 0, 0));
        assert_eq!(set.len(), 1);
        set.insert(Color::rgb(0, 255, 0));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_color_error_display() {
        let err = ColorError::InvalidHex("xyz".to_string());
        assert_eq!(format!("{err}"), "invalid hex color: xyz");
        let err = ColorError::InvalidFormat("bad".to_string());
        assert_eq!(format!("{err}"), "invalid color format: bad");
    }

    #[test]
    fn test_luminance() {
        let black = Color::rgb(0, 0, 0);
        assert!((black.luminance() - 0.0).abs() < 0.001);

        let white = Color::rgb(255, 255, 255);
        assert!((white.luminance() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_named_color_count() {
        assert!(NAMED_COLORS.len() >= 140);
    }

    #[test]
    fn test_gray_grey_alias() {
        assert_eq!(Color::named("gray"), Color::named("grey"));
        assert_eq!(Color::named("darkgray"), Color::named("darkgrey"));
    }

    #[test]
    fn test_from_hsv() {
        // Pure red in HSV is (0, 1, 1)
        let red = Color::from_hsv(0.0, 1.0, 1.0);
        assert_eq!(red, Color::rgb(255, 0, 0));
    }

    #[test]
    fn test_achromatic_hsl() {
        let gray = Color::rgb(128, 128, 128);
        let (_, s, _) = gray.to_hsl();
        assert!(s.abs() < 0.01);
    }
}

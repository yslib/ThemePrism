use std::fmt;

use palette::{FromColor, Hsl, LinSrgba, Mix, Srgb, Srgba};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self::new_rgba(r, g, b, 1.0)
    }

    pub const fn new_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }

    pub fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new_rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    pub fn from_hex(input: &str) -> Result<Self, String> {
        let hex = input.trim().trim_start_matches('#');
        if !matches!(hex.len(), 6 | 8) {
            return Err(format!("invalid hex color: {input}"));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| format!("invalid hex color: {input}"))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| format!("invalid hex color: {input}"))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| format!("invalid hex color: {input}"))?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).map_err(|_| format!("invalid hex color: {input}"))?
        } else {
            u8::MAX
        };
        Ok(Self::from_rgba_u8(r, g, b, a))
    }

    pub fn to_rgb_u8(self) -> (u8, u8, u8) {
        let clamp = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        (clamp(self.r), clamp(self.g), clamp(self.b))
    }

    pub fn to_rgba_u8(self) -> (u8, u8, u8, u8) {
        let clamp = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        (clamp(self.r), clamp(self.g), clamp(self.b), clamp(self.a))
    }

    pub fn to_hex(self) -> String {
        if self.a >= 0.999 {
            return self.to_opaque_hex();
        }

        let (r, g, b, a) = self.to_rgba_u8();
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }

    pub fn to_opaque_hex(self) -> String {
        let (r, g, b) = self.to_rgb_u8();
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    pub fn mix(self, other: Self, ratio: f32) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);
        let mixed = self.to_linear_srgba().mix(other.to_linear_srgba(), ratio);
        Self::from_linear_srgba(mixed)
    }

    pub fn lighten(self, amount: f32) -> Self {
        let (h, s, l, a) = self.to_hsla();
        Self::from_hsla(h, s, (l + amount).clamp(0.0, 1.0), a)
    }

    pub fn darken(self, amount: f32) -> Self {
        self.lighten(-amount)
    }

    pub fn saturate(self, amount: f32) -> Self {
        let (h, s, l, a) = self.to_hsla();
        Self::from_hsla(h, (s + amount).clamp(0.0, 1.0), l, a)
    }

    pub fn desaturate(self, amount: f32) -> Self {
        self.saturate(-amount)
    }

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        Self::from_hsla(h, s, l, 1.0)
    }

    pub fn from_hsla(mut h: f32, s: f32, l: f32, a: f32) -> Self {
        h = h.rem_euclid(360.0);
        let hsl = Hsl::new(h, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
        let srgb = Srgb::from_color(hsl);
        Self::from_srgba(Srgba::new(
            srgb.red,
            srgb.green,
            srgb.blue,
            a.clamp(0.0, 1.0),
        ))
    }

    pub fn to_hsl(self) -> (f32, f32, f32) {
        let (h, s, l, _) = self.to_hsla();
        (h, s, l)
    }

    pub fn to_hsla(self) -> (f32, f32, f32, f32) {
        let hsl = Hsl::from_color(self.to_srgb());
        (
            hsl.hue.into_degrees().rem_euclid(360.0),
            hsl.saturation,
            hsl.lightness,
            self.a.clamp(0.0, 1.0),
        )
    }

    pub fn approx_eq(self, other: Self) -> bool {
        (self.r - other.r).abs() < 0.001
            && (self.g - other.g).abs() < 0.001
            && (self.b - other.b).abs() < 0.001
            && (self.a - other.a).abs() < 0.001
    }

    pub fn with_alpha(self, alpha: f32) -> Self {
        Self::new_rgba(self.r, self.g, self.b, alpha.clamp(0.0, 1.0))
    }

    fn to_srgb(self) -> Srgb<f32> {
        Srgb::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
        )
    }

    fn to_srgba(self) -> Srgba<f32> {
        Srgba::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
            self.a.clamp(0.0, 1.0),
        )
    }

    fn from_srgba(color: Srgba<f32>) -> Self {
        Self::new_rgba(color.red, color.green, color.blue, color.alpha)
    }

    fn to_linear_srgba(self) -> LinSrgba<f32> {
        self.to_srgba().into_linear()
    }

    fn from_linear_srgba(color: LinSrgba<f32>) -> Self {
        Self::from_srgba(Srgba::from_linear(color))
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl From<Color> for ratatui::style::Color {
    fn from(value: Color) -> Self {
        let (r, g, b) = value.to_rgb_u8();
        Self::Rgb(r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn hsl_round_trip_stays_close() {
        let color = Color::from_hsl(205.0, 0.65, 0.53);
        let (h, s, l) = color.to_hsl();

        assert!((h - 205.0).abs() < 0.2);
        assert!((s - 0.65).abs() < 0.01);
        assert!((l - 0.53).abs() < 0.01);
    }

    #[test]
    fn hex_round_trip_stays_stable() {
        let color = Color::from_hex("#5DA5D9").unwrap();
        assert_eq!(color.to_hex(), "#5DA5D9");
    }

    #[test]
    fn rgba_hex_round_trip_stays_stable() {
        let color = Color::from_hex("#5DA5D980").unwrap();
        assert_eq!(color.to_hex(), "#5DA5D980");
    }

    #[test]
    fn mix_stays_within_expected_range() {
        let a = Color::from_hex("#224466").unwrap();
        let b = Color::from_hex("#88CCFF").unwrap();
        let mixed = a.mix(b, 0.5);

        assert!(mixed.r >= a.r.min(b.r) && mixed.r <= a.r.max(b.r));
        assert!(mixed.g >= a.g.min(b.g) && mixed.g <= a.g.max(b.g));
        assert!(mixed.b >= a.b.min(b.b) && mixed.b <= a.b.max(b.b));
    }

    #[test]
    fn alpha_is_preserved_through_hsl_adjustments() {
        let color = Color::from_hex("#33669980").unwrap();
        let lighter = color.lighten(0.1);

        assert!((lighter.a - color.a).abs() < 0.001);
    }
}

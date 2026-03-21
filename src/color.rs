use std::fmt;

use palette::{FromColor, Hsl, LinSrgb, Mix, Srgb};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }

    pub fn from_hex(input: &str) -> Result<Self, String> {
        let hex = input.trim().trim_start_matches('#');
        if hex.len() != 6 {
            return Err(format!("invalid hex color: {input}"));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| format!("invalid hex color: {input}"))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| format!("invalid hex color: {input}"))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| format!("invalid hex color: {input}"))?;
        Ok(Self::from_rgb_u8(r, g, b))
    }

    pub fn to_rgb_u8(self) -> (u8, u8, u8) {
        let clamp = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        (clamp(self.r), clamp(self.g), clamp(self.b))
    }

    pub fn to_hex(self) -> String {
        let (r, g, b) = self.to_rgb_u8();
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    pub fn mix(self, other: Self, ratio: f32) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);
        let mixed = self.to_linear_srgb().mix(other.to_linear_srgb(), ratio);
        Self::from_linear_srgb(mixed)
    }

    pub fn lighten(self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, s, (l + amount).clamp(0.0, 1.0))
    }

    pub fn darken(self, amount: f32) -> Self {
        self.lighten(-amount)
    }

    pub fn saturate(self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, (s + amount).clamp(0.0, 1.0), l)
    }

    pub fn desaturate(self, amount: f32) -> Self {
        self.saturate(-amount)
    }

    pub fn from_hsl(mut h: f32, s: f32, l: f32) -> Self {
        h = h.rem_euclid(360.0);
        let hsl = Hsl::new(h, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
        Self::from_srgb(Srgb::from_color(hsl))
    }

    pub fn to_hsl(self) -> (f32, f32, f32) {
        let hsl = Hsl::from_color(self.to_srgb());
        (
            hsl.hue.into_degrees().rem_euclid(360.0),
            hsl.saturation,
            hsl.lightness,
        )
    }

    pub fn approx_eq(self, other: Self) -> bool {
        (self.r - other.r).abs() < 0.001
            && (self.g - other.g).abs() < 0.001
            && (self.b - other.b).abs() < 0.001
    }

    fn to_srgb(self) -> Srgb<f32> {
        Srgb::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
        )
    }

    fn from_srgb(color: Srgb<f32>) -> Self {
        Self::new(color.red, color.green, color.blue)
    }

    fn to_linear_srgb(self) -> LinSrgb<f32> {
        self.to_srgb().into_linear()
    }

    fn from_linear_srgb(color: LinSrgb<f32>) -> Self {
        Self::from_srgb(Srgb::from_linear(color))
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
    fn mix_stays_within_expected_range() {
        let a = Color::from_hex("#224466").unwrap();
        let b = Color::from_hex("#88CCFF").unwrap();
        let mixed = a.mix(b, 0.5);

        assert!(mixed.r >= a.r.min(b.r) && mixed.r <= a.r.max(b.r));
        assert!(mixed.g >= a.g.min(b.g) && mixed.g <= a.g.max(b.g));
        assert!(mixed.b >= a.b.min(b.b) && mixed.b <= a.b.max(b.b));
    }
}

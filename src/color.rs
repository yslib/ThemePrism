use std::fmt;

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
        Self::new(
            self.r * (1.0 - ratio) + other.r * ratio,
            self.g * (1.0 - ratio) + other.g * ratio,
            self.b * (1.0 - ratio) + other.b * ratio,
        )
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
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);

        if s == 0.0 {
            return Self::new(l, l, l);
        }

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
        let m = l - c / 2.0;

        let (r1, g1, b1) = match h {
            h if (0.0..60.0).contains(&h) => (c, x, 0.0),
            h if (60.0..120.0).contains(&h) => (x, c, 0.0),
            h if (120.0..180.0).contains(&h) => (0.0, c, x),
            h if (180.0..240.0).contains(&h) => (0.0, x, c),
            h if (240.0..300.0).contains(&h) => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::new(r1 + m, g1 + m, b1 + m)
    }

    pub fn to_hsl(self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;
        let l = (max + min) / 2.0;

        if delta.abs() < f32::EPSILON {
            return (0.0, 0.0, l);
        }

        let s = delta / (1.0 - (2.0 * l - 1.0).abs());
        let h = if max == self.r {
            60.0 * (((self.g - self.b) / delta).rem_euclid(6.0))
        } else if max == self.g {
            60.0 * (((self.b - self.r) / delta) + 2.0)
        } else {
            60.0 * (((self.r - self.g) / delta) + 4.0)
        };

        (h, s, l)
    }

    pub fn approx_eq(self, other: Self) -> bool {
        (self.r - other.r).abs() < 0.001
            && (self.g - other.g).abs() < 0.001
            && (self.b - other.b).abs() < 0.001
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

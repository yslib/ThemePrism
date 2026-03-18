use std::collections::BTreeMap;

use crate::color::Color;
use crate::params::ThemeParams;
use crate::tokens::PaletteSlot;

#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub slots: BTreeMap<PaletteSlot, Color>,
}

impl Palette {
    pub fn get(&self, slot: PaletteSlot) -> Option<Color> {
        self.slots.get(&slot).copied()
    }
}

pub fn generate_palette(params: &ThemeParams) -> Palette {
    let mut slots = BTreeMap::new();
    let is_dark = params.background_lightness <= 0.5;
    let contrast_bias = (params.contrast - 0.2) / 0.8;

    let bg0 = Color::from_hsl(
        params.background_hue,
        params.background_saturation * 0.85,
        params.background_lightness,
    );
    let bg1 = shift_lightness(
        bg0,
        if is_dark {
            0.04 + contrast_bias * 0.05
        } else {
            -(0.04 + contrast_bias * 0.04)
        },
    )
    .saturate(0.01 * params.vibrancy);
    let bg2 = shift_lightness(
        bg0,
        if is_dark {
            0.09 + contrast_bias * 0.07
        } else {
            -(0.09 + contrast_bias * 0.06)
        },
    )
    .saturate(0.02 * params.vibrancy);

    let fg_hue = (params.background_hue + 8.0).rem_euclid(360.0);
    let fg_sat = (params.background_saturation * 0.35 + 0.03).clamp(0.0, 0.22);
    let fg0_light = if is_dark {
        (0.55 + contrast_bias * 0.15).clamp(0.45, 0.78)
    } else {
        (0.38 - contrast_bias * 0.10).clamp(0.18, 0.45)
    };
    let fg1_light = if is_dark {
        (0.80 + contrast_bias * 0.10).clamp(0.68, 0.95)
    } else {
        (0.24 - contrast_bias * 0.08).clamp(0.05, 0.34)
    };
    let fg2_light = if is_dark {
        (0.93 + contrast_bias * 0.05).clamp(0.80, 0.99)
    } else {
        (0.12 - contrast_bias * 0.06).clamp(0.02, 0.22)
    };

    let fg0 = Color::from_hsl(fg_hue, fg_sat, fg0_light);
    let fg1 = Color::from_hsl(fg_hue, fg_sat * 0.8, fg1_light);
    let fg2 = Color::from_hsl(fg_hue, fg_sat * 0.7, fg2_light);

    let accent_sat = (params.accent_saturation + params.vibrancy * 0.15).clamp(0.25, 0.92);
    let accent_light_base = if is_dark {
        params.accent_lightness.clamp(0.45, 0.82)
    } else {
        (params.accent_lightness - 0.12).clamp(0.28, 0.68)
    };
    let accent_specs = [
        (-160.0, 0.10, 0.00),
        (-70.0, 0.03, 0.03),
        (-28.0, 0.08, 0.01),
        (0.0, 0.00, 0.00),
        (38.0, 0.04, -0.02),
        (86.0, 0.02, 0.02),
    ];

    let accents = accent_specs.map(|(offset, sat_boost, light_shift)| {
        Color::from_hsl(
            (params.accent_hue + offset).rem_euclid(360.0),
            (accent_sat + sat_boost).clamp(0.0, 1.0),
            (accent_light_base + light_shift).clamp(0.0, 1.0),
        )
    });

    slots.insert(PaletteSlot::Bg0, bg0);
    slots.insert(PaletteSlot::Bg1, bg1);
    slots.insert(PaletteSlot::Bg2, bg2);
    slots.insert(PaletteSlot::Fg0, fg0);
    slots.insert(PaletteSlot::Fg1, fg1);
    slots.insert(PaletteSlot::Fg2, fg2);
    slots.insert(PaletteSlot::Accent0, accents[0]);
    slots.insert(PaletteSlot::Accent1, accents[1]);
    slots.insert(PaletteSlot::Accent2, accents[2]);
    slots.insert(PaletteSlot::Accent3, accents[3]);
    slots.insert(PaletteSlot::Accent4, accents[4]);
    slots.insert(PaletteSlot::Accent5, accents[5]);

    Palette { slots }
}

fn shift_lightness(color: Color, amount: f32) -> Color {
    if amount >= 0.0 {
        color.lighten(amount)
    } else {
        color.darken(-amount)
    }
}

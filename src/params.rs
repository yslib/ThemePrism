#[derive(Debug, Clone, PartialEq)]
pub struct ThemeParams {
    pub background_hue: f32,
    pub background_lightness: f32,
    pub background_saturation: f32,
    pub contrast: f32,
    pub accent_hue: f32,
    pub accent_saturation: f32,
    pub accent_lightness: f32,
    pub selection_mix: f32,
    pub vibrancy: f32,
}

impl Default for ThemeParams {
    fn default() -> Self {
        Self {
            background_hue: 220.0,
            background_lightness: 0.12,
            background_saturation: 0.08,
            contrast: 0.85,
            accent_hue: 205.0,
            accent_saturation: 0.65,
            accent_lightness: 0.62,
            selection_mix: 0.35,
            vibrancy: 0.50,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKey {
    BackgroundHue,
    BackgroundLightness,
    BackgroundSaturation,
    Contrast,
    AccentHue,
    AccentSaturation,
    AccentLightness,
    SelectionMix,
    Vibrancy,
}

impl ParamKey {
    pub const ALL: [Self; 9] = [
        Self::BackgroundHue,
        Self::BackgroundLightness,
        Self::BackgroundSaturation,
        Self::Contrast,
        Self::AccentHue,
        Self::AccentSaturation,
        Self::AccentLightness,
        Self::SelectionMix,
        Self::Vibrancy,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::BackgroundHue => "Background Hue",
            Self::BackgroundLightness => "Background Light",
            Self::BackgroundSaturation => "Background Sat",
            Self::Contrast => "Contrast",
            Self::AccentHue => "Accent Hue",
            Self::AccentSaturation => "Accent Sat",
            Self::AccentLightness => "Accent Light",
            Self::SelectionMix => "Selection Mix",
            Self::Vibrancy => "Vibrancy",
        }
    }

    pub const fn range(self) -> (f32, f32) {
        match self {
            Self::BackgroundHue | Self::AccentHue => (0.0, 360.0),
            Self::BackgroundLightness | Self::AccentLightness => (0.02, 0.96),
            Self::BackgroundSaturation
            | Self::AccentSaturation
            | Self::SelectionMix
            | Self::Vibrancy => (0.0, 1.0),
            Self::Contrast => (0.2, 1.0),
        }
    }

    pub const fn step(self) -> f32 {
        match self {
            Self::BackgroundHue | Self::AccentHue => 5.0,
            Self::BackgroundLightness
            | Self::BackgroundSaturation
            | Self::Contrast
            | Self::AccentSaturation
            | Self::AccentLightness
            | Self::SelectionMix
            | Self::Vibrancy => 0.02,
        }
    }

    pub fn get(self, params: &ThemeParams) -> f32 {
        match self {
            Self::BackgroundHue => params.background_hue,
            Self::BackgroundLightness => params.background_lightness,
            Self::BackgroundSaturation => params.background_saturation,
            Self::Contrast => params.contrast,
            Self::AccentHue => params.accent_hue,
            Self::AccentSaturation => params.accent_saturation,
            Self::AccentLightness => params.accent_lightness,
            Self::SelectionMix => params.selection_mix,
            Self::Vibrancy => params.vibrancy,
        }
    }

    pub fn set(self, params: &mut ThemeParams, value: f32) {
        match self {
            Self::BackgroundHue => params.background_hue = value.rem_euclid(360.0),
            Self::BackgroundLightness => params.background_lightness = value,
            Self::BackgroundSaturation => params.background_saturation = value,
            Self::Contrast => params.contrast = value,
            Self::AccentHue => params.accent_hue = value.rem_euclid(360.0),
            Self::AccentSaturation => params.accent_saturation = value,
            Self::AccentLightness => params.accent_lightness = value,
            Self::SelectionMix => params.selection_mix = value,
            Self::Vibrancy => params.vibrancy = value,
        }
        clamp_params(params);
    }

    pub fn adjust(self, params: &mut ThemeParams, direction: i32) {
        let value = self.get(params);
        let next = value + self.step() * direction as f32;
        let (min, max) = self.range();
        self.set(params, next.clamp(min, max));
    }

    pub fn format_value(self, params: &ThemeParams) -> String {
        let value = self.get(params);
        match self {
            Self::BackgroundHue | Self::AccentHue => format!("{value:>6.1}"),
            _ => format!("{:>6.0}%", value * 100.0),
        }
    }
}

pub fn clamp_params(params: &mut ThemeParams) {
    let clamp = |value: f32, min: f32, max: f32| value.clamp(min, max);
    params.background_hue = params.background_hue.rem_euclid(360.0);
    params.accent_hue = params.accent_hue.rem_euclid(360.0);
    params.background_lightness = clamp(params.background_lightness, 0.02, 0.96);
    params.background_saturation = clamp(params.background_saturation, 0.0, 1.0);
    params.contrast = clamp(params.contrast, 0.2, 1.0);
    params.accent_saturation = clamp(params.accent_saturation, 0.0, 1.0);
    params.accent_lightness = clamp(params.accent_lightness, 0.02, 0.96);
    params.selection_mix = clamp(params.selection_mix, 0.0, 1.0);
    params.vibrancy = clamp(params.vibrancy, 0.0, 1.0);
}

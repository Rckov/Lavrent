#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PowerPreference {
    LowPower,
    #[default]
    HighPerformance,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoreOptions {
    pub power_preference: PowerPreference,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DocumentSource {
    pub html: String,
    pub author_css: Vec<String>,
    pub base_resource_uri: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorScheme {
    #[default]
    Light,
    Dark,
}

impl DocumentSource {
    #[must_use]
    pub fn new(html: impl Into<String>) -> Self {
        Self {
            html: html.into(),
            author_css: Vec::new(),
            base_resource_uri: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
    pub color_scheme: ColorScheme,
}

impl Viewport {
    #[must_use]
    pub const fn new(width: f64, height: f64, scale_factor: f64) -> Self {
        Self {
            width,
            height,
            scale_factor,
            color_scheme: ColorScheme::Light,
        }
    }

    #[must_use]
    pub const fn with_color_scheme(mut self, color_scheme: ColorScheme) -> Self {
        self.color_scheme = color_scheme;
        self
    }
}

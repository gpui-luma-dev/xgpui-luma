#![allow(deprecated)]

use crate::theme::stylesheet::{CommonStylesheet, PaletteRole, SliderMotionStylesheet, TransitionSpec};
use std::collections::HashMap;

use std::sync::{Arc, OnceLock};

pub trait SliderTheme: Send + Sync {
    /// The slider's style (#433): the binding templates resolve (see [`resolve_slider`](super::resolve_slider)) and
    /// draw from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("slider")
    }
}

pub fn default_slider_theme() -> Arc<dyn SliderTheme> {
    static THEME: OnceLock<Arc<dyn SliderTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledSliderTheme::sdk())).clone()
}

/// Nonlinear slider presentation metrics, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderDecoration {
    pub dial_size: f32,
    pub track_radius: f32,
}

impl SliderMotionStylesheet {
    /// Construct native SDK slider motion specs in pure Rust without preloading TOML.
    pub fn native_default() -> Self {
        Self { thumb: Some(TransitionSpec::Timed { duration_ms: 200, easing: crate::motion::Easing::OutCubic }) }
    }
}

/// Construct native SDK slider palette recipe mappings in pure Rust without preloading TOML.
pub fn native_sdk_slider_recipes() -> HashMap<PaletteRole, PaletteRole> {
    use PaletteRole::*;
    HashMap::from([
        (StateDisabledBackground, StateDisabledBackground),
        (StateDisabledForeground, StateDisabledForeground),
        (StatePressedBackground, StatePressedBackground),
        (StateSelectedBackground, StateSelectedBackground),
        (StateSelectedForeground, StateSelectedForeground),
        (SurfacePanelBackground, SurfacePanelBackground),
        (SurfaceSubtleBackground, SurfaceSubtleBackground),
    ])
}

/// Native SDK slider motion and palette recipes. Its geometry is `slider.lss`.
pub fn define_sdk_slider(sheet: &mut CommonStylesheet) {
    sheet.motion.slider = SliderMotionStylesheet::native_default();
    sheet.recipes.0.insert("slider".into(), native_sdk_slider_recipes());
}

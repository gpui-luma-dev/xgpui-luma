#![allow(deprecated)]

use crate::theme::stylesheet::{CommonStylesheet, PaletteRole, SliderMotionStylesheet, TransitionSpec};
use std::collections::HashMap;

use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::theme::InteractionState;

#[deprecated(note = "slider templates draw from `SliderTheme::style()`; resolve it with `resolve_slider` (#433)")]
#[derive(Clone, Debug)]
pub struct SliderLook {
    pub thumb_border_width: f32,
    /// Captured radial layout.
    pub decoration: crate::theme::stylesheet::SliderDecoration,
    pub track_background: Hsla,
    /// The track's outline, over its filled and unfilled parts (linear sliders). A zero width draws none.
    pub track_border: Hsla,
    pub track_border_width: f32,
    pub fill_background: Hsla,
    /// The track's and the filled range's fades in this state: each part's resolved `opacity`
    /// (`:disabled { opacity }`), 1 when unset.
    pub track_opacity: f32,
    pub fill_opacity: f32,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: f32,
}

pub trait SliderTheme: Send + Sync {
    /// The slider's style (#433): the binding templates resolve (see [`resolve_slider`](super::resolve_slider)) and
    /// draw from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("slider")
    }

    /// The slider's look, built from [`Self::style`] for code that still reads a [`SliderLook`].
    #[deprecated(note = "templates draw from `style()`; resolve it with `resolve_slider` (#433)")]
    fn resolve(&self, state: InteractionState) -> SliderLook {
        SliderLook::from_resolved(&super::styled::resolve_slider(&self.style(), state))
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

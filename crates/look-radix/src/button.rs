//! Radix-owned button variants: `.lss` templates (`style/button.lss`) and the native palette behind [`button_look_for`].
#![allow(deprecated)] // Compatibility paths and parity tests still read the deprecated Looks (#433).

use std::sync::Arc;

use xgpui_luma::controls::button::{ButtonData, ButtonTemplate, StyledButtonTemplate};

use crate::button_layout::{ButtonSize, Radius};
use crate::look::Look;
#[cfg(test)]
use crate::scale::ScaleFamily;
#[cfg(test)]
use crate::semantic::SemanticRole;
use crate::tone::Tone;

/// Look-owned button treatments. Not Shadcn variant names and not SDK enums.
///
/// [`Classic`](Self::Classic) through [`Ghost`](Self::Ghost) are the Radix Themes
/// variant scheme. The remaining variants are app chrome kept for one-off use and
/// are deliberately outside that scheme.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    /// Solid accent fill with a darker edge and a raised drop shadow.
    Classic,
    #[default]
    Solid,
    Soft,
    /// Accent 2 panel with an accent 7 edge and accent 11 text.
    Surface,
    Outline,
    Ghost,
    /// Bare chrome: no background, border, or focus ring in any interaction state.
    GhostQuiet,
    /// Screen-nav pill. Selected reads as a filled tab rather than a button.
    /// Interaction is binary (off | on) — no focus ring, hover, or pressed paint.
    Page,
}

/// Paint axes that sit beside [`ButtonVariant`]: accent vs gray, and Radix `highContrast`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Paint {
    pub tone: Tone,
    pub high_contrast: bool,
}

impl Paint {
    pub const fn accent() -> Self {
        Self { tone: Tone::Accent, high_contrast: false }
    }

    pub const fn gray() -> Self {
        Self { tone: Tone::Gray, high_contrast: false }
    }

    pub const fn high_contrast(mut self) -> Self {
        self.high_contrast = true;
        self
    }
}

/// The `.lss` template for a Radix variant × paint × size × radius cell (All Sizes matrix / previews, and every
/// button with its own size and radius): its faces carry the cell's box. Classic (the look's classic radius)
/// and page (a pill) take the corner of the solid cell instead.
pub fn button_cell_template<D: ButtonData>(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
    size: ButtonSize,
    radius: Radius,
) -> StyledButtonTemplate<D> {
    fn with_radius<T: ButtonData>(template: StyledButtonTemplate<T>, radius: Radius) -> StyledButtonTemplate<T> {
        // Medium is the base radius.
        if radius != Radius::Medium {
            template.attr("radius", radius.as_str())
        } else {
            template
        }
    }
    let template = with_radius(styled_button_template::<D>(look, variant, paint, size), radius);
    if matches!(variant, ButtonVariant::Classic | ButtonVariant::Page) {
        use xgpui_luma::controls::button_family::ButtonFamilyRole;
        use xgpui_luma::style::PartRead;
        use xgpui_luma::style::lss::{Overrides, PartId, Prop};
        let solid = with_radius(styled_button_template::<()>(look, ButtonVariant::Solid, paint, size), radius);
        let corner = |role| {
            solid.resolve_face(role, Default::default(), 1.0).root().number(if role == ButtonFamilyRole::Icon {
                "icon_radius"
            } else {
                "radius"
            })
        };
        return template.overrides(
            Overrides::new().set(PartId::ROOT, Prop::<f32>::new("radius"), corner(ButtonFamilyRole::Text)).set(
                PartId::ROOT,
                Prop::<f32>::new("icon_radius"),
                corner(ButtonFamilyRole::Icon),
            ),
        );
    }
    template
}

/// Full look resolver for a Radix size × radius cell.
#[deprecated(note = "use `button_cell_template` and read its faces (#433)")]
pub fn button_look_for<D: 'static>(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
    size: ButtonSize,
    radius: Radius,
) -> xgpui_luma::controls::button::ButtonLookSource<D> {
    let template = button_cell_template::<()>(look, variant, paint, size, radius);
    Arc::new(move |model| template.look(model.role, model.state, 1.0))
}

/// Tunable geometry for the [`Classic`](ButtonVariant::Classic) button.
///
/// Held on [`Look`](crate::Look) so a tuning UI can drive every Classic
/// button live. Defaults are sampled from a reference render: the face is a vertical
/// wash that grows *lighter* toward the bottom, a 1px specular rim sits just inside
/// the top border, and a single border darker than the fill wraps the whole edge.
///
/// `*_delta` and `rim_lighten` are lightness offsets applied to the accent fill,
/// `*_alpha` are `0..1`, and the rest are `px`.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassicButtonParams {
    /// Pressed rim shading relative to the resting highlight.
    pub pressed_rim_lightness_scale: f32,
    pub pressed_rim_alpha_scale: f32,
    /// Preserve pigment near black and white while shifting lightness.
    pub lightness_min: f32,
    pub lightness_max: f32,
    /// Corner radius for every Classic button, replacing the size-scale radius. The
    /// rim shadows trace this curve, so the two have to be tuned together.
    pub radius: f32,
    /// Lightness offset at the top of the face wash.
    pub fill_top_delta: f32,
    /// Lightness offset at the bottom of the face wash. Positive: bottom is lighter.
    pub fill_bottom_delta: f32,
    /// Lightness offset for the border that wraps the edge. Negative: darker than fill.
    pub border_delta: f32,
    /// Lightness dropped below the fill for the shaded lip inside the bottom border.
    /// Reads as a second, darker edge band under the face.
    pub bevel_darken: f32,
    pub bevel_offset: f32,
    pub bevel_alpha: f32,
    /// Lightness offset for the specular highlight inside the top border.
    pub rim_lighten: f32,
    pub rim_offset: f32,
    pub rim_alpha: f32,
    pub fade_offset: f32,
    pub fade_blur: f32,
    /// Negative spread keeps the fade a top band instead of a full-perimeter halo.
    pub fade_spread: f32,
    pub fade_alpha: f32,
    pub drop_offset: f32,
    pub drop_blur: f32,
    pub drop_alpha: f32,
}

impl ClassicButtonParams {
    pub(crate) fn zero() -> Self {
        Self {
            pressed_rim_lightness_scale: 0.0,
            pressed_rim_alpha_scale: 0.0,
            lightness_min: 0.0,
            lightness_max: 1.0,
            radius: 0.0,
            fill_top_delta: 0.0,
            fill_bottom_delta: 0.0,
            border_delta: 0.0,
            bevel_darken: 0.0,
            bevel_offset: 0.0,
            bevel_alpha: 0.0,
            rim_lighten: 0.0,
            rim_offset: 0.0,
            rim_alpha: 0.0,
            fade_offset: 0.0,
            fade_blur: 0.0,
            fade_spread: 0.0,
            fade_alpha: 0.0,
            drop_offset: 0.0,
            drop_blur: 0.0,
            drop_alpha: 0.0,
        }
    }
}

impl Default for ClassicButtonParams {
    /// The bundled Classic bubble, read by `button.lss` as `look.classic.*`.
    fn default() -> Self {
        let mut classic = Self::zero();
        classic.pressed_rim_lightness_scale = 0.5;
        classic.pressed_rim_alpha_scale = 0.6;
        classic.lightness_min = 0.02;
        classic.lightness_max = 0.98;
        classic.radius = 4.0;
        classic.fill_top_delta = 0.0;
        classic.fill_bottom_delta = 0.067;
        classic.border_delta = -0.022;
        classic.bevel_darken = 0.045;
        classic.bevel_offset = 1.0;
        classic.bevel_alpha = 1.0;
        classic.rim_lighten = 0.3;
        classic.rim_offset = 1.0;
        classic.rim_alpha = 0.95;
        classic.fade_offset = 4.0;
        classic.fade_blur = 4.0;
        classic.fade_spread = -2.0;
        classic.fade_alpha = 0.35;
        classic.drop_offset = 1.0;
        classic.drop_blur = 2.0;
        classic.drop_alpha = 0.08;
        classic
    }
}

/// Template for a Radix variant × paint cell at the look's size, drawn from `.lss` (`style/button.lss`).
pub fn button_template<D: ButtonData>(look: &Look, variant: ButtonVariant, paint: Paint) -> Arc<dyn ButtonTemplate<D>> {
    Arc::new(button_template_for::<D>(look, variant, paint))
}

/// [`button_template`] as the `.lss` template itself, for callers that add modifiers or read its faces.
pub fn button_template_for<D: ButtonData>(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
) -> StyledButtonTemplate<D> {
    let size = match look.size() {
        crate::RadixSize::One => ButtonSize::One,
        crate::RadixSize::Two => ButtonSize::Two,
        crate::RadixSize::Three => ButtonSize::Three,
    };
    styled_button_template::<D>(look, variant, paint, size)
}

/// The classic variant (its gradient face comes from `background_start` / `background_end`).
pub fn classic_button_template<D: ButtonData>(look: &Look) -> Arc<dyn ButtonTemplate<D>> {
    button_template(look, ButtonVariant::Classic, Paint::accent())
}

/// Classic template with an explicit paint axis (tone / high contrast).
pub fn classic_button_template_with<D: ButtonData>(look: &Look, paint: Paint) -> Arc<dyn ButtonTemplate<D>> {
    button_template(look, ButtonVariant::Classic, paint)
}

/// A `.lss` button-family template (`button`, or `toggle` for [`ToggleData`]) for one variant × paint × size cell.
pub(crate) fn styled_button_template<D: ButtonData>(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
    size: ButtonSize,
) -> StyledButtonTemplate<D> {
    let source = look.clone();
    let mut template = StyledButtonTemplate::<D>::live(Arc::new(move || source.style_theme()))
        .state("high_contrast", paint.high_contrast);
    let variant = match variant {
        ButtonVariant::Solid => None,
        ButtonVariant::Classic => Some("classic"),
        ButtonVariant::Soft => Some("soft"),
        ButtonVariant::Surface => Some("surface"),
        ButtonVariant::Outline => Some("outline"),
        ButtonVariant::Ghost => Some("ghost"),
        ButtonVariant::GhostQuiet => Some("ghost_quiet"),
        ButtonVariant::Page => Some("page"),
    };
    if let Some(variant) = variant {
        template = template.variant(variant);
    }
    if size != ButtonSize::Two {
        template = template.attr("size", size.as_str());
    }
    if paint.tone == Tone::Gray {
        template = template.attr("tone", "gray");
    }
    template
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{BoxShadow, Hsla, px};
    use xgpui_luma::controls::button_family::ButtonFamilyRole;
    use xgpui_luma::theme::InteractionState;

    /// Equal up to float rounding (colors and channels).
    fn assert_close_any<T: std::fmt::Debug + PartialEq>(a: T, b: T) {
        let (sa, sb) = (format!("{a:?}"), format!("{b:?}"));
        let nums = |s: &str| -> Vec<f32> {
            s.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == 'e'))
                .filter_map(|n| n.parse().ok())
                .collect()
        };
        let (na, nb) = (nums(&sa), nums(&sb));
        assert!(na.len() == nb.len() && na.iter().zip(&nb).all(|(x, y)| (x - y).abs() < 2e-3), "{sa} vs {sb}");
    }

    /// The native palette for a variant × paint cell (what [`button_look_for`] paints).
    struct Native {
        look: Look,
        variant: ButtonVariant,
        paint: Paint,
    }

    impl Native {
        /// The cell's `.lss` face (what [`button_look_for`] paints).
        fn resolve(
            &self,
            role: ButtonFamilyRole,
            state: InteractionState,
        ) -> xgpui_luma::controls::button_family::ButtonFamilyLook {
            let source = button_look_for::<()>(&self.look, self.variant, self.paint, ButtonSize::Two, Radius::Medium);
            source(&xgpui_luma::controls::button::ButtonRenderModel::<()> { role, state, ..Default::default() })
        }
    }

    fn native(look: &Look, variant: ButtonVariant) -> Native {
        native_with(look, variant, Paint::accent())
    }

    fn native_with(look: &Look, variant: ButtonVariant, paint: Paint) -> Native {
        Native { look: look.clone(), variant, paint }
    }

    #[test]
    fn solid_default_uses_primary_steps() {
        let look = Look::built_in();
        let theme = native(&look, ButtonVariant::Solid);
        let palette = theme.resolve(ButtonFamilyRole::Text, InteractionState::default());
        let expected_bg = look.resolve_role(SemanticRole::Primary).hsla();
        assert_close_any(palette.background.h, expected_bg.h);
        assert_close_any(palette.background.l, expected_bg.l);
    }

    #[test]
    fn ghost_default_is_transparent() {
        let look = Look::built_in();
        let theme = native(&look, ButtonVariant::Ghost);
        let palette = theme.resolve(ButtonFamilyRole::Text, InteractionState::default());
        assert_close_any(palette.background.a, 0.0);
        assert!(palette.border.is_none());
    }

    #[test]
    fn surface_uses_accent_panel_and_edge() {
        let look = Look::built_in();
        let theme = native(&look, ButtonVariant::Surface);
        let palette = theme.resolve(ButtonFamilyRole::Text, InteractionState::default());

        assert_close_any(palette.background, look.resolve_step(ScaleFamily::Color, 2).hsla());
        assert_close_any(palette.border, Some(look.resolve_step(ScaleFamily::Color, 7).hsla()));
        assert_close_any(palette.foreground, look.resolve_step(ScaleFamily::Color, 11).hsla());
    }

    #[test]
    fn solid_high_contrast_uses_step_twelve_on_gray_one() {
        let look = Look::built_in();
        let theme = native_with(&look, ButtonVariant::Solid, Paint::gray().high_contrast());
        let palette = theme.resolve(ButtonFamilyRole::Text, InteractionState::default());

        assert_close_any(palette.background, Tone::Gray.step(&look, 12));
        assert_close_any(palette.foreground, look.resolve_step(ScaleFamily::Gray, 1).hsla());
    }

    #[test]
    fn classic_high_contrast_uses_step_twelve() {
        let look = Look::built_in();
        let theme = native_with(&look, ButtonVariant::Classic, Paint::accent().high_contrast());
        let palette = theme.resolve(ButtonFamilyRole::Text, InteractionState::default());

        assert_close_any(palette.background, Tone::Accent.step(&look, 12));
        assert_close_any(palette.foreground, look.resolve_step(ScaleFamily::Gray, 1).hsla());
    }

    #[test]
    fn soft_high_contrast_darkens_the_label_only() {
        let look = Look::built_in();
        let normal = native_with(&look, ButtonVariant::Soft, Paint::gray());
        let hc = native_with(&look, ButtonVariant::Soft, Paint::gray().high_contrast());
        let state = InteractionState::default();
        let normal = normal.resolve(ButtonFamilyRole::Text, state);
        let hc = hc.resolve(ButtonFamilyRole::Text, state);

        assert_close_any(hc.background, normal.background);
        assert_close_any(hc.foreground, Tone::Gray.step(&look, 12));
        assert_ne!(hc.foreground, normal.foreground);
    }

    #[test]
    fn classic_high_contrast_shifts_fill_to_step_twelve() {
        let look = Look::built_in();
        let normal = native_with(&look, ButtonVariant::Classic, Paint::gray());
        let hc = native_with(&look, ButtonVariant::Classic, Paint::gray().high_contrast());
        let state = InteractionState::default();

        assert_ne!(
            normal.resolve(ButtonFamilyRole::Text, state).background,
            hc.resolve(ButtonFamilyRole::Text, state).background
        );
    }

    #[test]
    fn classic_lime_uses_dark_contrast_label() {
        let look = Look::from_palettes(
            crate::palette::Accent::Lime,
            crate::palette::Gray::Auto,
            Default::default(),
            xgpui_luma::theme::ThemeMode::Light,
        );
        let theme = native_with(&look, ButtonVariant::Classic, Paint::accent());
        let palette = theme.resolve(ButtonFamilyRole::Text, InteractionState::default());

        assert!(palette.foreground.l < palette.background.l);
        assert_close_any(palette.foreground, look.resolve_role(SemanticRole::PrimaryForeground).hsla());
    }

    #[test]
    fn classic_shares_the_solid_fill_behind_a_darker_border() {
        let look = Look::built_in();
        let classic = native(&look, ButtonVariant::Classic);
        let solid = native(&look, ButtonVariant::Solid);
        let state = InteractionState::default();
        let palette = classic.resolve(ButtonFamilyRole::Text, state);

        assert_close_any(palette.background, solid.resolve(ButtonFamilyRole::Text, state).background);
        let border = palette.border.expect("classic keeps a border that delineates the edge");
        assert!(border.l < palette.background.l, "border is darker than the fill");
        assert_close_any(border.h, palette.background.h);
    }

    /// The classic face's `.lss` shadows.
    fn classic_shadows(look: &Look, state: InteractionState) -> Vec<BoxShadow> {
        native(look, ButtonVariant::Classic)
            .resolve(ButtonFamilyRole::Text, state)
            .shadow
            .unwrap_or_default()
    }

    #[test]
    fn classic_bevels_a_lit_top_and_a_shaded_bottom() {
        let look = Look::built_in();
        let fill = look.resolve_role(SemanticRole::Primary).hsla();
        let shadows = classic_shadows(&look, InteractionState::default());
        let edge = |downward: bool| {
            shadows
                .iter()
                .filter(|shadow| shadow.inset && (shadow.offset.y > px(0.0)) == downward)
                .collect::<Vec<_>>()
        };
        let (top, bottom) = (edge(true), edge(false));

        assert!(top.iter().all(|shadow| shadow.color.l > fill.l), "the top rim is lighter than the fill");
        assert!(!bottom.is_empty(), "the bottom keeps a shaded lip");
        assert!(bottom.iter().all(|shadow| shadow.color.l < fill.l), "the bottom lip is darker than the fill");
        assert!(
            shadows.iter().all(|shadow| !shadow.inset || (shadow.color.h - fill.h).abs() < 1e-3),
            "edges keep the accent hue"
        );
        assert!(top.iter().any(|shadow| shadow.blur_radius == px(0.0)), "has a crisp specular line");
        // Negative spread keeps the fade a top band instead of a full-perimeter halo.
        assert!(
            top.iter().any(|shadow| shadow.blur_radius > px(0.0) && shadow.spread_radius < px(0.0)),
            "has an inward-pulled fade"
        );
    }

    #[test]
    fn classic_face_lightens_toward_the_bottom() {
        use xgpui_luma::style::lss::{AttrValue, ControlId, StyleKey, Value};
        let look = Look::built_in();
        let config = crate::style::config();
        let theme = look.style_theme();
        let wash = |pressed: bool| {
            let key = StyleKey::new(ControlId::new("button"))
                .attr_opt(config.attribute("variant").unwrap(), Some(AttrValue::new("classic")))
                .state_if(config.state("pressed").unwrap(), pressed);
            let resolved = theme.resolve(&key);
            let stop = |k: &str| Hsla::from(resolved.root().value(k).and_then(Value::as_color).unwrap());
            (stop("background_start"), stop("background_end"))
        };
        let ((top, bottom), (pressed_top, pressed_bottom)) = (wash(false), wash(true));

        assert!(bottom.l > top.l, "the wash lifts toward the bottom, away from the overhead light");
        assert!(pressed_bottom.l < pressed_top.l, "pressing flips the wash so the face reads recessed");
    }

    #[test]
    fn classic_shadow_follows_the_look_params() {
        let look = Look::built_in();
        let blur = || classic_shadows(&look, InteractionState::default()).iter().any(|s| s.blur_radius == px(19.0));

        assert!(!blur());
        look.set_classic_params(ClassicButtonParams { fade_blur: 19.0, ..Default::default() });
        assert!(blur(), "editor retunes live buttons");
    }

    #[test]
    fn page_toggle_stays_borderless_when_unselected() {
        let look = Look::built_in();
        let theme = native(&look, ButtonVariant::Page);
        let palette = theme.resolve(ButtonFamilyRole::Toggle { selected: false }, InteractionState::default());

        assert_close_any(palette.background.a, 0.0);
        assert!(palette.border.is_none());
    }

    #[test]
    fn page_toggle_is_binary_off_on() {
        let look = Look::built_in();
        let theme = native(&look, ButtonVariant::Page);
        let role = ButtonFamilyRole::Toggle { selected: true };
        let resting = theme.resolve(role, InteractionState::default());
        let focused = theme.resolve(role, InteractionState { focused: true, ..Default::default() });
        let pressed = theme.resolve(role, InteractionState { hovered: true, pressed: true, ..Default::default() });
        let hovered = theme.resolve(role, InteractionState { hovered: true, ..Default::default() });

        assert!(resting.border.is_none(), "Page face is borderless");
        assert_close_any(resting.focus_border.a, 0.0);
        assert_close_any(focused.border, resting.border);
        assert_close_any(focused.background, resting.background);
        assert_close_any(pressed.background, resting.background);
        assert_close_any(hovered.background, resting.background);
    }

    #[test]
    fn ghost_quiet_has_no_background_or_border_in_any_state() {
        let look = Look::built_in();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for paint in [Paint::accent(), Paint::gray(), Paint { tone: Tone::Accent, high_contrast: true }] {
                let quiet = native_with(&look, ButtonVariant::GhostQuiet, paint);
                for role in
                    [ButtonFamilyRole::Text, ButtonFamilyRole::Icon, ButtonFamilyRole::Toggle { selected: true }]
                {
                    for bits in 0..16 {
                        let state = InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            ..Default::default()
                        };
                        let palette = quiet.resolve(role, state);
                        assert_close_any(palette.background.a, 0.0);
                        assert!(palette.border.is_none(), "{mode:?} {role:?} {state:?}");
                    }
                }
            }
        }
        let ghost = native(&look, ButtonVariant::Ghost);
        assert!(
            ghost
                .resolve(ButtonFamilyRole::Text, InteractionState { hovered: true, ..Default::default() })
                .background
                .a
                > 0.0
        );
    }
}

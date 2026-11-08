//! Radix-owned button variants bound to SDK [`ButtonFamilyTheme`].

use std::sync::Arc;

use gpui::{Background, BoxShadow, Hsla, black, linear_color_stop, linear_gradient, point, prelude::*, px};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate};
use xgpui_luma::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, ButtonFamilyTheme, compose_button_family_look,
};
use xgpui_luma::theme::{ControlSize, InteractionState, MetricTokens, StandardBoxScale};

use crate::button_layout::{ButtonSize, Radius, apply_button_box, button_box_with_stylesheet};
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

struct ButtonFamilyThemeAdapter {
    look: Look,
    variant: ButtonVariant,
    paint: Paint,
    size: Option<ButtonSize>,
}

impl ButtonFamilyTheme for ButtonFamilyThemeAdapter {
    fn resolve(&self, role: ButtonFamilyRole, _size: ControlSize, state: InteractionState) -> ButtonFamilyPalette {
        resolve_button_palette(&self.look, self.variant, self.paint, role, state)
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn resolve_look(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
        scale: &StandardBoxScale,
        pill_radius: f32,
    ) -> Option<ButtonFamilyLook> {
        let mut palette = self.resolve(role, size, state);
        let geometry = self.look.stylesheet_snapshot().button.resolve_geometry(
            self.size.map(ButtonSize::as_str).unwrap_or_else(|| crate::look::sdk_size_key(size)),
            xgpui_luma::theme::stylesheet::ButtonGeometry {
                height: scale.height,
                padding_x: scale.padding_x,
                padding_y: scale.padding_y,
                gap: scale.gap,
                icon_size: scale.icon_size,
                font_size: palette.typography.size,
                line_height: palette.typography.line_height,
            },
        );
        palette.typography.size = geometry.font_size;
        palette.typography.line_height = geometry.line_height;
        let mut look = compose_button_family_look(&palette, role, scale, pill_radius);
        look.height = geometry.height;
        look.gap = geometry.gap;
        if role != ButtonFamilyRole::Icon {
            look.padding_x = geometry.padding_x;
            look.padding_y = geometry.padding_y;
        } else {
            look.icon_size = geometry.icon_size;
        }
        if matches!(self.variant, ButtonVariant::Page) {
            look.radius = pill_radius;
        }
        if matches!(self.variant, ButtonVariant::Classic) {
            look.shadow = Some(classic_shadow(&self.look, palette.background, state));
            // Classic keeps an editable radius knob; default matches Radix size-2 / medium (4px).
            look.radius = self.look.classic_params().radius;
        }
        Some(look)
    }
}

/// Accent tone, normal contrast — the default Radix button paint.
pub fn button_family_theme(look: &Look, variant: ButtonVariant) -> Arc<dyn ButtonFamilyTheme> {
    button_family_theme_with(look, variant, Paint::accent())
}

/// Full look resolver for a Radix size × radius cell (All Sizes matrix / previews).
pub fn button_look_for<D: 'static>(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
    size: ButtonSize,
    radius: Radius,
) -> xgpui_luma::controls::button::ButtonLookSource<D> {
    let look = look.clone();
    Arc::new(move |model| {
        let palette = resolve_button_palette(&look, variant, paint, model.role, model.state);
        let mut family = apply_button_box(
            &palette,
            model.role,
            button_box_with_stylesheet(&look.stylesheet_snapshot(), size, radius),
        );
        if matches!(variant, ButtonVariant::Classic) {
            family.shadow = Some(classic_shadow(&look, family.background, model.state));
        }
        family
    })
}

pub fn button_family_theme_with(look: &Look, variant: ButtonVariant, paint: Paint) -> Arc<dyn ButtonFamilyTheme> {
    Arc::new(ButtonFamilyThemeAdapter { look: look.clone(), variant, paint, size: None })
}

pub(crate) fn button_family_theme_for_size(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
    size: ButtonSize,
) -> Arc<dyn ButtonFamilyTheme> {
    Arc::new(ButtonFamilyThemeAdapter { look: look.clone(), variant, paint, size: Some(size) })
}

fn resolve_button_palette(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonFamilyPalette {
    let colors = super::button_recipes::resolve(look, variant, paint, role, state);
    let stylesheet = look.stylesheet_snapshot();
    let tokens = &stylesheet.tokens.typography;
    ButtonFamilyPalette {
        background: colors.background,
        foreground: colors.foreground,
        muted_foreground: colors.foreground,
        border: colors.border,
        typography: tokens.text.label,
        font_family: tokens.font.sans.family.clone().into(),
    }
}

fn inset_shadow(offset_y: f32, blur: f32, spread: f32, color: Hsla) -> BoxShadow {
    BoxShadow {
        offset: point(px(0.0), px(offset_y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        color,
        inset: true,
    }
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
    fn default() -> Self {
        super::button_recipes::embedded().classic
    }
}

#[cfg(test)]
pub(crate) fn shift_lightness(color: Hsla, delta: f32) -> Hsla {
    shift_lightness_with_params(color, delta, super::button_recipes::embedded().classic)
}

pub(crate) fn shift_lightness_with_params(color: Hsla, delta: f32, params: ClassicButtonParams) -> Hsla {
    Hsla { l: (color.l + delta).clamp(params.lightness_min, params.lightness_max), ..color }
}

/// Vertical wash stops for the Classic face: the light source is overhead, so the top
/// sits at the accent and the bottom lifts. Pressing flips them so the face reads
/// recessed.
fn classic_wash(params: ClassicButtonParams, base: Hsla, pressed: bool) -> (Hsla, Hsla) {
    let top = shift_lightness_with_params(base, params.fill_top_delta, params);
    let bottom = shift_lightness_with_params(base, params.fill_bottom_delta, params);
    if pressed { (bottom, top) } else { (top, bottom) }
}

fn classic_fill(look: &Look, paint: Paint, role: ButtonFamilyRole, state: InteractionState) -> Background {
    let base = resolve_button_palette(look, ButtonVariant::Classic, paint, role, state).background;
    if state.disabled {
        return base.into();
    }

    let (top, bottom) = classic_wash(look.classic_params(), base, state.pressed);
    linear_gradient(180.0, linear_color_stop(top, 0.0), linear_color_stop(bottom, 1.0))
}

/// Bevel for the Classic face: a crisp specular rim just inside the top border with a
/// short fade below it, and a shaded lip inside the bottom border. Pressing drops the
/// lip and inverts the rim so the light reads as coming from below.
fn classic_shadow(look: &Look, fill: Hsla, state: InteractionState) -> Vec<BoxShadow> {
    let params = look.classic_params();
    let tint = |delta: f32, alpha: f32| Hsla { a: alpha, ..shift_lightness_with_params(fill, delta, params) };

    if state.pressed {
        let shade = -params.rim_lighten * params.pressed_rim_lightness_scale;
        return vec![
            inset_shadow(params.rim_offset, 0.0, 0.0, tint(shade, params.rim_alpha * params.pressed_rim_alpha_scale)),
            inset_shadow(params.fade_offset, params.fade_blur, params.fade_spread, tint(shade, params.fade_alpha)),
        ];
    }

    let mut shadows = vec![
        inset_shadow(params.rim_offset, 0.0, 0.0, tint(params.rim_lighten, params.rim_alpha)),
        inset_shadow(
            params.fade_offset,
            params.fade_blur,
            params.fade_spread,
            tint(params.rim_lighten, params.fade_alpha),
        ),
        inset_shadow(-params.bevel_offset, 0.0, 0.0, tint(-params.bevel_darken, params.bevel_alpha)),
    ];
    if params.drop_alpha > 0.0 {
        shadows.push(BoxShadow {
            offset: point(px(0.0), px(params.drop_offset)),
            blur_radius: px(params.drop_blur),
            spread_radius: px(0.0),
            color: Hsla { a: params.drop_alpha, ..black() },
            inset: false,
        });
    }
    shadows
}

/// Template for a Radix variant × paint cell. Classic keeps its gradient wash.
pub fn button_template<D: Clone + 'static>(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<D>> {
    if matches!(variant, ButtonVariant::Classic) {
        classic_button_template_with(look, paint)
    } else {
        Arc::new(DefaultButtonTemplate::new(button_family_theme_with(look, variant, paint)))
    }
}

/// Classic needs a gradient face, which [`ButtonFamilyLook`] cannot carry, so the look
/// ships its own template. Callers that build templates by hand (the style guide matrix)
/// must use this instead of a bare [`DefaultButtonTemplate`] or they lose the wash.
pub fn classic_button_template<D: Clone + 'static>(look: &Look) -> Arc<dyn ButtonTemplate<D>> {
    classic_button_template_with(look, Paint::accent())
}

/// Classic template with an explicit paint axis (tone / high contrast).
pub fn classic_button_template_with<D: Clone + 'static>(look: &Look, paint: Paint) -> Arc<dyn ButtonTemplate<D>> {
    let theme = button_family_theme_with(look, ButtonVariant::Classic, paint);
    let look = look.clone();
    Arc::new(DefaultButtonTemplate::new(theme).with_modifier(move |root, model: &ButtonRenderModel<D>| {
        root.bg(classic_fill(&look, paint, model.role, model.state))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::InteractionState;

    #[test]
    fn solid_default_uses_primary_steps() {
        let look = Look::built_in();
        let theme = button_family_theme(&look, ButtonVariant::Solid);
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());
        let expected_bg = look.resolve_role(SemanticRole::Primary).hsla();
        assert_eq!(palette.background.h, expected_bg.h);
        assert_eq!(palette.background.l, expected_bg.l);
    }

    #[test]
    fn ghost_default_is_transparent() {
        let look = Look::built_in();
        let theme = button_family_theme(&look, ButtonVariant::Ghost);
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());
        assert_eq!(palette.background.a, 0.0);
        assert!(palette.border.is_none());
    }

    #[test]
    fn surface_uses_accent_panel_and_edge() {
        let look = Look::built_in();
        let theme = button_family_theme(&look, ButtonVariant::Surface);
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());

        assert_eq!(palette.background, look.resolve_step(ScaleFamily::Color, 2).hsla());
        assert_eq!(palette.border, Some(look.resolve_step(ScaleFamily::Color, 7).hsla()));
        assert_eq!(palette.foreground, look.resolve_step(ScaleFamily::Color, 11).hsla());
    }

    #[test]
    fn solid_high_contrast_uses_step_twelve_on_gray_one() {
        let look = Look::built_in();
        let theme = button_family_theme_with(&look, ButtonVariant::Solid, Paint::gray().high_contrast());
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());

        assert_eq!(palette.background, Tone::Gray.step(&look, 12));
        assert_eq!(palette.foreground, look.resolve_step(ScaleFamily::Gray, 1).hsla());
    }

    #[test]
    fn classic_high_contrast_uses_step_twelve() {
        let look = Look::built_in();
        let theme = button_family_theme_with(&look, ButtonVariant::Classic, Paint::accent().high_contrast());
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());

        assert_eq!(palette.background, Tone::Accent.step(&look, 12));
        assert_eq!(palette.foreground, look.resolve_step(ScaleFamily::Gray, 1).hsla());
    }

    #[test]
    fn soft_high_contrast_darkens_the_label_only() {
        let look = Look::built_in();
        let normal = button_family_theme_with(&look, ButtonVariant::Soft, Paint::gray());
        let hc = button_family_theme_with(&look, ButtonVariant::Soft, Paint::gray().high_contrast());
        let state = InteractionState::default();
        let normal = normal.resolve(ButtonFamilyRole::Text, ControlSize::Md, state);
        let hc = hc.resolve(ButtonFamilyRole::Text, ControlSize::Md, state);

        assert_eq!(hc.background, normal.background);
        assert_eq!(hc.foreground, Tone::Gray.step(&look, 12));
        assert_ne!(hc.foreground, normal.foreground);
    }

    #[test]
    fn classic_high_contrast_shifts_fill_to_step_twelve() {
        let look = Look::built_in();
        let normal = button_family_theme_with(&look, ButtonVariant::Classic, Paint::gray());
        let hc = button_family_theme_with(&look, ButtonVariant::Classic, Paint::gray().high_contrast());
        let state = InteractionState::default();

        assert_ne!(
            normal.resolve(ButtonFamilyRole::Text, ControlSize::Md, state).background,
            hc.resolve(ButtonFamilyRole::Text, ControlSize::Md, state).background
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
        let theme = button_family_theme_with(&look, ButtonVariant::Classic, Paint::accent());
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());

        assert!(palette.foreground.l < palette.background.l);
        assert_eq!(palette.foreground, look.resolve_role(SemanticRole::PrimaryForeground).hsla());
    }

    #[test]
    fn classic_shares_the_solid_fill_behind_a_darker_border() {
        let look = Look::built_in();
        let classic = button_family_theme(&look, ButtonVariant::Classic);
        let solid = button_family_theme(&look, ButtonVariant::Solid);
        let state = InteractionState::default();
        let palette = classic.resolve(ButtonFamilyRole::Text, ControlSize::Md, state);

        assert_eq!(palette.background, solid.resolve(ButtonFamilyRole::Text, ControlSize::Md, state).background);
        let border = palette.border.expect("classic keeps a border that delineates the edge");
        assert!(border.l < palette.background.l, "border is darker than the fill");
        assert_eq!(border.h, palette.background.h, "border keeps the accent hue");
    }

    fn classic_shadows(look: &Look, state: InteractionState) -> Vec<BoxShadow> {
        let theme = button_family_theme(&look, ButtonVariant::Classic);
        let scale = StandardBoxScale::compute(ControlSize::Md, &look.metrics(), 2.0);
        theme
            .resolve_look(ButtonFamilyRole::Text, ControlSize::Md, state, &scale, 999.0)
            .expect("classic look")
            .shadow
            .expect("classic shadows")
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
        assert!(shadows.iter().all(|shadow| !shadow.inset || shadow.color.h == fill.h), "edges keep the accent hue");
        assert!(top.iter().any(|shadow| shadow.blur_radius == px(0.0)), "has a crisp specular line");
        // Negative spread keeps the fade a top band instead of a full-perimeter halo.
        assert!(
            top.iter().any(|shadow| shadow.blur_radius > px(0.0) && shadow.spread_radius < px(0.0)),
            "has an inward-pulled fade"
        );
    }

    #[test]
    fn classic_face_lightens_toward_the_bottom() {
        let look = Look::built_in();
        let fill = look.resolve_role(SemanticRole::Primary).hsla();
        let params = look.classic_params();
        let (top, bottom) = classic_wash(params, fill, false);
        let (pressed_top, pressed_bottom) = classic_wash(params, fill, true);

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
        let theme = button_family_theme(&look, ButtonVariant::Page);
        let palette =
            theme.resolve(ButtonFamilyRole::Toggle { selected: false }, ControlSize::Md, InteractionState::default());

        assert_eq!(palette.background.a, 0.0);
        assert!(palette.border.is_none());
    }

    #[test]
    fn page_toggle_is_binary_off_on() {
        let look = Look::built_in();
        let theme = button_family_theme(&look, ButtonVariant::Page);
        let role = ButtonFamilyRole::Toggle { selected: true };
        let resting = theme.resolve(role, ControlSize::Md, InteractionState::default());
        let focused = theme.resolve(role, ControlSize::Md, InteractionState { focused: true, ..Default::default() });
        let pressed = theme.resolve(
            role,
            ControlSize::Md,
            InteractionState { hovered: true, pressed: true, ..Default::default() },
        );
        let hovered = theme.resolve(role, ControlSize::Md, InteractionState { hovered: true, ..Default::default() });

        assert!(resting.border.is_none(), "opaque page borders become SDK focus rings");
        assert_eq!(focused.border, resting.border);
        assert_eq!(focused.background, resting.background);
        assert_eq!(pressed.background, resting.background);
        assert_eq!(hovered.background, resting.background);
    }

    #[test]
    fn ghost_quiet_has_no_background_or_border_in_any_state() {
        let look = Look::built_in();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for paint in [Paint::accent(), Paint::gray(), Paint { tone: Tone::Accent, high_contrast: true }] {
                let quiet = button_family_theme_with(&look, ButtonVariant::GhostQuiet, paint);
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
                        let palette = quiet.resolve(role, ControlSize::Md, state);
                        assert_eq!(palette.background.a, 0.0, "{mode:?} {role:?} {state:?}");
                        assert!(palette.border.is_none(), "{mode:?} {role:?} {state:?}");
                    }
                }
            }
        }
        let ghost = button_family_theme(&look, ButtonVariant::Ghost);
        assert!(
            ghost
                .resolve(
                    ButtonFamilyRole::Text,
                    ControlSize::Md,
                    InteractionState { hovered: true, ..Default::default() }
                )
                .background
                .a
                > 0.0
        );
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_button_stylesheet() -> xgpui_luma::theme::stylesheet::ButtonStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ButtonStylesheet {
        geometry: ButtonGeometryStylesheet { padding_y: Some(0.0), ..Default::default() },
        sizes: [
            ("1", 24.0, 8.0, 4.0, 12.0, 16.0, 12.0),
            ("2", 32.0, 12.0, 8.0, 14.0, 20.0, 14.0),
            ("3", 40.0, 16.0, 12.0, 16.0, 24.0, 16.0),
            ("4", 48.0, 24.0, 12.0, 18.0, 26.0, 18.0),
        ]
        .into_iter()
        .map(|(key, height, padding, gap, font, line, icon)| {
            (
                key.into(),
                ButtonGeometryStylesheet {
                    height: Some(height),
                    padding_x: Some(padding),
                    gap: Some(gap),
                    font_size: Some(font),
                    line_height: Some(line),
                    icon_size: Some(icon),
                    ..Default::default()
                },
            )
        })
        .collect(),
    }
}

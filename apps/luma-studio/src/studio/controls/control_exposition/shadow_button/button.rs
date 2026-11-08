use std::sync::Arc;

use gpui::{App, BoxShadow, Div, Hsla, SharedString, Stateful, Window, div, point, prelude::*, px};
use xgpui_luma::controls::button_family::{ButtonFamilyLook, button_family_effective_border};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate, button_content_context};
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_shadcn::ShadcnLook;

const DEFAULT_OFFSET_X: f32 = 0.0;
const DEFAULT_OFFSET_Y: f32 = 5.0;
const DEFAULT_BLUR: f32 = 5.0;
const DEFAULT_SPREAD: f32 = 0.0;
const DEFAULT_OPACITY: f32 = 0.35;
const DISABLED_OPACITY: f32 = 0.56;

const PRESSED_OFFSET_FACTOR: f32 = 0.55;
const PRESSED_OPACITY_FACTOR: f32 = 0.82;
const HOVER_OFFSET_BONUS: f32 = 1.0;
const HOVER_OPACITY_BONUS: f32 = 0.03;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ButtonVisualState {
    Default,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

impl From<InteractionState> for ButtonVisualState {
    fn from(state: InteractionState) -> Self {
        if state.disabled {
            Self::Disabled
        } else if state.pressed {
            Self::Pressed
        } else if state.hovered {
            Self::Hovered
        } else if state.focused {
            Self::Focused
        } else {
            Self::Default
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ShadowButtonSpec {
    pub color: Hsla,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub opacity: f32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ShadowButtonControls {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub opacity: f32,
}

impl Default for ShadowButtonControls {
    fn default() -> Self {
        Self {
            offset_x: DEFAULT_OFFSET_X,
            offset_y: DEFAULT_OFFSET_Y,
            blur_radius: DEFAULT_BLUR,
            spread_radius: DEFAULT_SPREAD,
            opacity: DEFAULT_OPACITY,
        }
    }
}

impl ShadowButtonControls {
    pub(super) fn base_spec(self, look: &ShadcnLook) -> ShadowButtonSpec {
        ShadowButtonSpec {
            color: resolved_shadow_color(look),
            offset_x: self.offset_x,
            offset_y: self.offset_y,
            blur_radius: self.blur_radius,
            spread_radius: self.spread_radius,
            opacity: self.opacity,
        }
    }
}

#[derive(Clone)]
struct PrototypeShadowButtonTemplate {
    controls: ShadowButtonControls,
    color_override: Option<Hsla>,
    look: Arc<ShadcnLook>,
}

impl ButtonTemplate<()> for PrototypeShadowButtonTemplate {
    fn render(&self, model: &ButtonRenderModel<()>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let look = self.resolve_look(model);
        let border = button_family_effective_border(look.border);

        let mut control = div()
            .id(format!("{}-control", model.id))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(look.gap))
            .bg(look.background)
            .text_color(look.foreground)
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .font_family(look.font_family.clone())
            .font_weight(look.typography.weight)
            .h(px(look.height));

        if border.a > 0.0 {
            control = control.border_1().border_color(border);
        }

        if model.round {
            control = control.w(px(look.height)).p_0().rounded_full();
        } else {
            control = control.px(px(look.padding_x)).py(px(look.padding_y)).rounded(px(look.radius));
        }

        let content_model = button_content_context(model, look.clone());
        control = control.child(div().text_color(look.foreground).child((model.content)(&content_model, cx)));

        let radius = if let Some(radius_override) = model.radius_override.get() {
            radius_override
        } else if model.round {
            look.height / 2.0
        } else {
            look.radius
        };

        let state = ButtonVisualState::from(model.state);
        let mut base_spec = self.controls.base_spec(&self.look);
        if let Some(color) = self.color_override {
            base_spec.color = color;
        }
        let shadow_spec = resolve_shadow_spec(base_spec, state);
        let shadow_insets = shadow_spec.map(shadow_projection_insets).unwrap_or_default();

        let root_insets = shadow_insets;

        let mut surface = div().id(format!("{}-surface", model.id)).relative();
        if let Some(spec) = shadow_spec {
            surface = surface.child(render_shadow(&model.id, spec, radius));
        }
        surface = surface.child(control);

        let adorned = div().id(format!("{}-adorned", model.id)).relative().child(surface);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .pt(px(root_insets.top))
            .pr(px(root_insets.right))
            .pb(px(root_insets.bottom))
            .pl(px(root_insets.left))
            .child(adorned);

        if model.state.disabled {
            root = root.opacity(DISABLED_OPACITY);
        } else {
            root = root.cursor_pointer();
        }

        let _ = window;
        root
    }
}

impl PrototypeShadowButtonTemplate {
    fn resolve_look(&self, model: &ButtonRenderModel<()>) -> ButtonFamilyLook {
        if let Some(resolve) = &model.look {
            resolve(model)
        } else {
            self.look.resolve_primary_button(model.role, model.size, model.state)
        }
    }
}

pub(super) fn prototype_shadow_button_template(
    look: Arc<ShadcnLook>,
    controls: ShadowButtonControls,
    color_override: Option<Hsla>,
) -> Arc<dyn ButtonTemplate<()>> {
    Arc::new(PrototypeShadowButtonTemplate { controls, color_override, look })
}

pub(super) fn resolve_shadow_spec(base: ShadowButtonSpec, state: ButtonVisualState) -> Option<ShadowButtonSpec> {
    match state {
        ButtonVisualState::Disabled => None,
        ButtonVisualState::Hovered => Some(ShadowButtonSpec {
            offset_y: base.offset_y + HOVER_OFFSET_BONUS,
            opacity: (base.opacity + HOVER_OPACITY_BONUS).clamp(0.0, 1.0),
            ..base
        }),
        ButtonVisualState::Pressed => Some(ShadowButtonSpec {
            offset_y: (base.offset_y * PRESSED_OFFSET_FACTOR).max(1.0),
            opacity: (base.opacity * PRESSED_OPACITY_FACTOR).clamp(0.0, 1.0),
            ..base
        }),
        ButtonVisualState::Default | ButtonVisualState::Focused => Some(base),
    }
}

pub(super) fn resolved_shadow_color(look: &ShadcnLook) -> Hsla {
    let chrome = look.chrome();
    let background = chrome.panel_background;

    if background.l <= 0.35 {
        Hsla { h: 0.0, s: 0.0, l: 0.82, a: 1.0 }
    } else {
        Hsla { h: 0.0, s: 0.0, l: 0.08, a: 1.0 }
    }
}

pub(super) fn default_shadow_color_placeholder(color: Hsla) -> String {
    let (r, g, b) = rgb_triplet(color);
    format!("{r}, {g}, {b}")
}

pub(super) fn format_css_box_shadow(spec: ShadowButtonSpec) -> String {
    format!(
        "box-shadow: {} {}px {}px {}px {}px;",
        format_css_rgba(spec.color, spec.opacity),
        rounded_px(spec.offset_x),
        rounded_px(spec.offset_y),
        rounded_px(spec.blur_radius.max(0.0)),
        rounded_px(spec.spread_radius),
    )
}

pub(super) fn format_rgb_triplet(color: Hsla) -> String {
    let (r, g, b) = rgb_triplet(color);
    format!("{r}, {g}, {b}")
}

fn render_shadow(id: &SharedString, spec: ShadowButtonSpec, radius: f32) -> Stateful<Div> {
    let insets = shadow_projection_insets(spec);

    div()
        .id(format!("{}-shadow", id))
        .absolute()
        .top(px(-insets.top))
        .left(px(-insets.left))
        .right(px(-insets.right))
        .bottom(px(-insets.bottom))
        .rounded(px(radius))
        .bg(spec.color.opacity(0.0))
        .shadow(shadow_layers(spec))
}

fn format_css_rgba(color: Hsla, opacity: f32) -> String {
    let (r, g, b) = rgb_triplet(color);
    format!("rgba({r}, {g}, {b}, {})", compact_alpha(opacity.clamp(0.0, 1.0)))
}

fn rounded_px(value: f32) -> i32 {
    value.round() as i32
}

fn rgb_triplet(color: Hsla) -> (u8, u8, u8) {
    let h = color.h.fract() * 6.0;
    let s = color.s.clamp(0.0, 1.0);
    let l = color.l.clamp(0.0, 1.0);

    if s <= f32::EPSILON {
        let gray = (l * 255.0).round() as u8;
        return (gray, gray, gray);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let r = hue_to_channel(p, q, h + 0.0);
    let g = hue_to_channel(p, q, h + 2.0);
    let b = hue_to_channel(p, q, h + 4.0);

    ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
}

fn compact_alpha(value: f32) -> String {
    let formatted = format!("{value:.3}");
    formatted.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn shadow_layers(spec: ShadowButtonSpec) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: spec.color.opacity(spec.opacity.clamp(0.0, 1.0)),
        offset: point(px(spec.offset_x), px(spec.offset_y)),
        blur_radius: px(spec.blur_radius.max(0.0)),
        spread_radius: px(spec.spread_radius),
        inset: false,
    }]
}

#[derive(Clone, Copy, Debug)]
struct ShadowProjectionInsets {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl Default for ShadowProjectionInsets {
    fn default() -> Self {
        Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 }
    }
}

fn shadow_projection_insets(spec: ShadowButtonSpec) -> ShadowProjectionInsets {
    let reach = (spec.blur_radius.max(0.0) + spec.spread_radius).max(0.0);

    ShadowProjectionInsets {
        top: (reach - spec.offset_y).ceil().max(0.0),
        right: (reach + spec.offset_x).ceil().max(0.0),
        bottom: (reach + spec.offset_y).ceil().max(0.0),
        left: (reach - spec.offset_x).ceil().max(0.0),
    }
}

fn hue_to_channel(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t += 6.0;
    }
    if t >= 6.0 {
        t -= 6.0;
    }
    if t < 1.0 {
        p + (q - p) * t
    } else if t < 3.0 {
        q
    } else if t < 4.0 {
        p + (q - p) * (4.0 - t)
    } else {
        p
    }
}

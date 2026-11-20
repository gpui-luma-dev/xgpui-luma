use xgpui_luma::theme::InteractionLayer;

use crate::controls::ShadcnButtonStyle;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::config::{ButtonColorRule, ScrollbarColorRule, SidebarContainerColorRule, SliderColorRule};

use super::{ResolveContext, ResolvedFields, resolve_color_ref, resolve_stylesheet_color};

pub fn resolve_button_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ButtonColorRule,
) -> anyhow::Result<ResolvedButtonColors> {
    let mut fields = ResolvedFields::default();

    let background = resolve_color_ref(resolver, &rule.background, &fields)?;
    fields.insert("background", background.clone());

    let foreground = resolve_color_ref(resolver, &rule.foreground, &fields)?;
    fields.insert("foreground", foreground.clone());

    let border = match &rule.border {
        Some(value) => Some(resolve_color_ref(resolver, value, &fields)?),
        None => None,
    };

    Ok(ResolvedButtonColors { background, foreground, border })
}

pub fn resolve_slider_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SliderColorRule,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedSliderColors> {
    let ctx = ResolveContext { style, layer, disabled: layer == InteractionLayer::Disabled };
    let mut fields = ResolvedFields::default();
    let track_background = resolve_stylesheet_color(resolver, &rule.track_background, &fields, &ctx)?;
    fields.insert("track_background", track_background.clone());
    let fill_background = resolve_stylesheet_color(resolver, &rule.fill_background, &fields, &ctx)?;
    fields.insert("fill_background", fill_background.clone());
    let thumb_background = resolve_stylesheet_color(resolver, &rule.thumb_background, &fields, &ctx)?;
    fields.insert("thumb_background", thumb_background.clone());
    let thumb_border = resolve_stylesheet_color(resolver, &rule.thumb_border, &fields, &ctx)?;
    Ok(ResolvedSliderColors { track_background, fill_background, thumb_background, thumb_border })
}

pub fn resolve_scrollbar_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ScrollbarColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedScrollbarColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let track_background = resolve_stylesheet_color(resolver, &rule.track_background, &fields, &ctx)?;
    fields.insert("track_background", track_background.clone());
    let thumb_background = resolve_stylesheet_color(resolver, &rule.thumb_background, &fields, &ctx)?;
    Ok(ResolvedScrollbarColors { track_background, thumb_background })
}

pub fn resolve_sidebar_container_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SidebarContainerColorRule,
) -> anyhow::Result<ResolvedSidebarContainerColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    Ok(ResolvedSidebarContainerColors { background, foreground, border })
}

#[derive(Clone, Debug)]
pub struct ResolvedStepperMetrics {
    pub step_badge_size: f32,
    pub track_thickness: f32,
}

#[derive(Clone, Debug)]
pub struct ResolvedSliderColors {
    pub track_background: ResolvedColor,
    pub fill_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedScrollbarColors {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedSidebarContainerColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedSliderMetrics {
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedScrollbarMetrics {
    pub thickness: f32,
    pub track_thickness: f32,
    pub thumb_thickness: f32,
    pub min_thumb_length: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedProgressMetrics {
    pub size: f32,
    pub stroke_width: f32,
    pub track_height: f32,
    pub thumb_size: f32,
}

#[derive(Clone, Debug)]
pub struct ResolvedButtonColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedButtonMetrics {
    pub height: f32,
    pub padding_horizontal: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub corner_radius: f32,
}

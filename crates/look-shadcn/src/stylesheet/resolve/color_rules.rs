use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::config::{ButtonColorRule, SidebarContainerColorRule};

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
pub struct ResolvedSidebarContainerColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
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

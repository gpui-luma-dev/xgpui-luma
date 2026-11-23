use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::config::SidebarContainerColorRule;

use super::{ResolveContext, ResolvedFields, resolve_stylesheet_color};

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

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedButtonMetrics {
    pub height: f32,
    pub padding_horizontal: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub corner_radius: f32,
}

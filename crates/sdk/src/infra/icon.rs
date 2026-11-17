use gpui::{AnyElement, Hsla, SharedString, Transformation, px, prelude::*, radians, svg};
use lucide_svg_static::Icon as LucideIcon;
use lucide_svg_static::Icon as SvgIcon;

#[derive(Clone, Debug, PartialEq)]
pub enum IconSource {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}

/// The two icons used by an expanding or opening disclosure affordance.
///
/// The defaults preserve the SDK's existing Lucide appearance. Consumers can
/// replace either side independently with a Lucide glyph or an SVG path.
#[derive(Clone, Debug)]
pub struct DisclosureIcons {
    pub expanded: IconSource,
    pub collapsed: IconSource,
}

/// Locale-aware standard right/down disclosure; custom icon pairs keep their artwork.
pub(crate) fn render_logical_disclosure_icon(
    icons: &DisclosureIcons,
    progress: f32,
    color: Hsla,
    size: f32,
    direction: crate::infra::locale::LayoutDirection,
) -> AnyElement {
    if direction == crate::infra::locale::LayoutDirection::Rtl
        && icons.collapsed.lucide() == Some(LucideIcon::ChevronRight)
        && icons.expanded.lucide() == Some(LucideIcon::ChevronDown)
    {
        return svg()
            .path(SvgIcon::ChevronLeft.asset_path())
            .size(px(size))
            .text_color(color)
            .with_transformation(Transformation::rotate(radians(
                -progress.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2,
            )))
            .into_any_element();
    }
    render_disclosure_icon(icons, progress, color, size)
}

/// Renders a disclosure affordance with a continuous rotation for the standard
/// chevron pairs, while preserving icon swapping for custom icon combinations.
pub fn render_disclosure_icon(icons: &DisclosureIcons, progress: f32, color: Hsla, size: f32) -> AnyElement {
    let progress = progress.clamp(0.0, 1.0);
    let animated_pair = match (&icons.collapsed, &icons.expanded) {
        (IconSource::Lucide(LucideIcon::ChevronRight), IconSource::Lucide(LucideIcon::ChevronDown)) => {
            Some((SvgIcon::ChevronRight, std::f32::consts::FRAC_PI_2))
        }
        (IconSource::Lucide(LucideIcon::ChevronDown), IconSource::Lucide(LucideIcon::ChevronUp)) => {
            Some((SvgIcon::ChevronDown, std::f32::consts::PI))
        }
        _ => None,
    };

    if let Some((asset, angle)) = animated_pair {
        return svg()
            .path(asset.asset_path())
            .size(px(size))
            .text_color(color)
            .with_transformation(Transformation::rotate(radians(progress * angle)))
            .into_any_element();
    }

    render_icon_source(
        if progress >= 0.5 {
            &icons.expanded
        } else {
            &icons.collapsed
        },
        color,
        size,
    )
}

impl DisclosureIcons {
    /// Resolve the standard disclosure pair from a selected stylesheet.
    pub fn from_stylesheet(stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        use crate::theme::stylesheet::IconRole;
        Self::new(
            stylesheet.icons.resolve(IconRole::DisclosureExpanded),
            stylesheet.icons.resolve(IconRole::DisclosureCollapsed),
        )
    }
    /// Resolve the opening/closing pair used by popup triggers.
    pub fn menu_from_stylesheet(stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        use crate::theme::stylesheet::IconRole;
        Self::new(stylesheet.icons.resolve(IconRole::MenuExpanded), stylesheet.icons.resolve(IconRole::MenuCollapsed))
    }
    pub fn new(expanded: impl Into<IconSource>, collapsed: impl Into<IconSource>) -> Self {
        Self { expanded: expanded.into(), collapsed: collapsed.into() }
    }
}

impl Default for DisclosureIcons {
    fn default() -> Self {
        let icons = &crate::theme::stylesheet::CommonStylesheet::sdk_default().icons;
        Self::new(
            icons.resolve(crate::theme::stylesheet::IconRole::DisclosureExpanded),
            icons.resolve(crate::theme::stylesheet::IconRole::DisclosureCollapsed),
        )
    }
}

/// Selection and completion affordances shared by choice and progress controls.
#[derive(Clone, Debug)]
pub struct SelectionStatusIcons {
    pub selected: IconSource,
    pub completed: IconSource,
}

impl Default for SelectionStatusIcons {
    fn default() -> Self {
        Self::from_stylesheet(crate::theme::stylesheet::CommonStylesheet::sdk_default())
    }
}

impl SelectionStatusIcons {
    /// Resolve shared selection/completion icons before consumer overrides.
    pub fn from_stylesheet(stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        use crate::theme::stylesheet::IconRole;
        Self {
            selected: stylesheet.icons.resolve(IconRole::Selected),
            completed: stylesheet.icons.resolve(IconRole::Completed),
        }
    }
}

impl IconSource {
    pub fn lucide(&self) -> Option<LucideIcon> {
        match self {
            Self::Lucide(icon) => Some(*icon),
            Self::SvgPath(_) => None,
        }
    }

    pub fn svg_path(&self) -> Option<&SharedString> {
        match self {
            Self::Lucide(_) => None,
            Self::SvgPath(path) => Some(path),
        }
    }
}

impl From<LucideIcon> for IconSource {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

impl From<&str> for IconSource {
    fn from(icon: &str) -> Self {
        Self::SvgPath(SharedString::from(icon))
    }
}

impl From<String> for IconSource {
    fn from(icon: String) -> Self {
        Self::SvgPath(icon.into())
    }
}

impl From<SharedString> for IconSource {
    fn from(icon: SharedString) -> Self {
        Self::SvgPath(icon)
    }
}

pub fn lucide_icon(icon: LucideIcon, color: Hsla, size: f32) -> AnyElement {
    svg().size(px(size)).path(icon.asset_path()).text_color(color).into_any_element()
}

pub fn render_icon_source(icon: &IconSource, color: Hsla, size: f32) -> AnyElement {
    match icon {
        IconSource::Lucide(icon) => lucide_icon(*icon, color, size),
        IconSource::SvgPath(path) => {
            svg().size(px(size)).text_color(color).external_path(path.clone()).into_any_element()
        }
    }
}

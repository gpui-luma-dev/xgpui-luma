use gpui::Div;
use gpui::IntoElement;
use gpui::SharedString;
use gpui::div;
use gpui::px;
use gpui::prelude::*;
use xgpui_luma::infra::icon::IconSource;
use xgpui_luma::infra::icon::lucide_icon;

use crate::look::ShadcnLook;
use crate::size::ShadcnSize;
use crate::provenance::ColorSource;
use crate::provenance::ResolvedColor;
use crate::tokens::ShadcnToken;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Outline,
    Ghost,
}

impl BadgeVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Secondary => "secondary",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BadgeIconPlacement {
    #[default]
    Start,
    End,
}

#[derive(Clone)]
pub struct Badge {
    look: ShadcnLook,
    label: SharedString,
    variant: BadgeVariant,
    size: ShadcnSize,
    icon: Option<IconSource>,
    icon_placement: BadgeIconPlacement,
}

#[derive(Clone, Debug)]
pub struct BadgeColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

pub use xgpui_luma::controls::badge::BadgeLook;

impl BadgeColorTable {
    pub fn fallback(look: &ShadcnLook, variant: BadgeVariant) -> Self {
        let (background, foreground, border) = match variant {
            BadgeVariant::Default => (
                ResolvedColor {
                    value: look.color(ShadcnToken::Primary),
                    source: ColorSource::Derived { note: "badge fallback primary".into() },
                },
                ResolvedColor {
                    value: look.color(ShadcnToken::PrimaryForeground),
                    source: ColorSource::Derived { note: "badge fallback primary-foreground".into() },
                },
                None,
            ),
            BadgeVariant::Secondary => (
                ResolvedColor {
                    value: look.color(ShadcnToken::Secondary),
                    source: ColorSource::Derived { note: "badge fallback secondary".into() },
                },
                ResolvedColor {
                    value: look.color(ShadcnToken::SecondaryForeground),
                    source: ColorSource::Derived { note: "badge fallback secondary-foreground".into() },
                },
                None,
            ),
            BadgeVariant::Outline => (
                ResolvedColor::transparent(),
                ResolvedColor {
                    value: look.color(ShadcnToken::Foreground),
                    source: ColorSource::Derived { note: "badge fallback foreground".into() },
                },
                Some(ResolvedColor {
                    value: look.color(ShadcnToken::Border),
                    source: ColorSource::Derived { note: "badge fallback border".into() },
                }),
            ),
            BadgeVariant::Ghost => (
                ResolvedColor::transparent(),
                ResolvedColor {
                    value: look.color(ShadcnToken::Foreground),
                    source: ColorSource::Derived { note: "badge fallback foreground".into() },
                },
                None,
            ),
        };

        Self { background, foreground, border }
    }
}

impl Badge {
    pub fn new(look: ShadcnLook, label: impl Into<SharedString>) -> Self {
        Self {
            look,
            label: label.into(),
            variant: BadgeVariant::Default,
            size: ShadcnSize::Md,
            icon: None,
            icon_placement: BadgeIconPlacement::Start,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn icon(mut self, icon: impl Into<IconSource>, placement: BadgeIconPlacement) -> Self {
        self.icon = Some(icon.into());
        self.icon_placement = placement;
        self
    }

    pub fn start_icon(self, icon: impl Into<IconSource>) -> Self {
        self.icon(icon, BadgeIconPlacement::Start)
    }

    pub fn end_icon(self, icon: impl Into<IconSource>) -> Self {
        self.icon(icon, BadgeIconPlacement::End)
    }
}

/// The badge from `.lss` (`style/badge.lss`) for `variant` at `size`.
pub fn badge_look(theme: &ShadcnLook, variant: BadgeVariant, size: ShadcnSize) -> BadgeLook {
    let look = theme.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "badge",
        xgpui_luma::style::ThemeSource::Live(std::sync::Arc::new(move || look.style_theme())),
    );
    let binding = if size != ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    let binding = if variant != BadgeVariant::Default {
        binding.variant(variant.as_str())
    } else {
        binding
    };
    xgpui_luma::controls::badge::StyledBadge::new(binding).look()
}

impl IntoElement for Badge {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let look = badge_look(&self.look, self.variant, self.size);
        let mut root = div()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(look.gap))
            .min_h(px(look.min_height))
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .rounded(px(look.radius))
            .bg(look.background)
            .text_color(look.foreground)
            .font_family(look.font_family)
            .font_weight(look.typography.weight)
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .when_some(look.border, |root, border| root.border_1().border_color(border));

        if let Some(icon) = self.icon.as_ref() {
            let rendered = render_badge_icon(icon, look.foreground, look.icon_size);
            if self.icon_placement == BadgeIconPlacement::Start {
                root = root.child(rendered).child(self.label);
            } else {
                root = root.child(self.label).child(rendered);
            }
        } else {
            root = root.child(self.label);
        }

        root
    }
}

fn render_badge_icon(icon: &IconSource, color: gpui::Hsla, size: f32) -> gpui::AnyElement {
    if let Some(icon) = icon.lucide() {
        return lucide_icon(icon, color, size);
    }

    if let Some(path) = icon.svg_path() {
        return gpui::svg().external_path(path.clone()).size(px(size)).text_color(color).into_any_element();
    }

    div().size(px(size)).into_any_element()
}

impl ShadcnLook {
    pub fn badge(&self, label: impl Into<SharedString>) -> Badge {
        Badge::new(self.clone(), label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_look() -> ShadcnLook {
        crate::test_support::fallback_look()
    }

    #[test]
    fn default_badge_uses_primary_tokens() {
        let shadcn = sample_look();
        let look = badge_look(&shadcn, BadgeVariant::Default, ShadcnSize::Md);

        crate::assert_close(look.background, shadcn.color(ShadcnToken::Primary));
        crate::assert_close(look.foreground, shadcn.color(ShadcnToken::PrimaryForeground));
    }

    #[test]
    fn outline_badge_uses_border_in_dark_mode() {
        let shadcn = sample_look();
        shadcn.set_mode(xgpui_luma::theme::ThemeMode::Dark);
        let look = badge_look(&shadcn, BadgeVariant::Outline, ShadcnSize::Md);

        crate::assert_close(look.foreground, shadcn.color(ShadcnToken::Foreground));
        crate::assert_close(look.border.expect("outline border"), shadcn.token_color("input").expect("input"));
    }

    #[test]
    fn ghost_badge_is_transparent() {
        let shadcn = sample_look();
        let look = badge_look(&shadcn, BadgeVariant::Ghost, ShadcnSize::Md);

        assert_eq!(look.background.a, 0.0);
        assert!(look.border.is_none());
    }

    #[test]
    fn badge_size_resolves_button_typography() {
        let shadcn = sample_look();
        let small = badge_look(&shadcn, BadgeVariant::Secondary, ShadcnSize::Sm);
        let medium = badge_look(&shadcn, BadgeVariant::Secondary, ShadcnSize::Md);
        let large = badge_look(&shadcn, BadgeVariant::Secondary, ShadcnSize::Lg);

        assert!(small.typography.size < medium.typography.size);
        assert!(medium.typography.size < large.typography.size);
        assert_eq!(small.icon_size, small.typography.size);
        assert_eq!(large.icon_size, large.typography.size);
    }
}

#![allow(deprecated)] // Still reads a deprecated Look until its family has a style-override replacement (#433).
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

pub use xgpui_luma::controls::badge::BadgeLook;

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

/// The badge's `.lss` (`style/badge.lss`) for `variant` at `size` (#433): the badge draws from its resolved root.
pub fn badge_style(
    theme: &ShadcnLook,
    variant: BadgeVariant,
    size: ShadcnSize,
) -> xgpui_luma::controls::badge::StyledBadge {
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
    xgpui_luma::controls::badge::StyledBadge::new(binding)
}

/// The badge from `.lss` (`style/badge.lss`) for `variant` at `size`.
#[deprecated(note = "the badge draws from `badge_style` (#433)")]
pub fn badge_look(theme: &ShadcnLook, variant: BadgeVariant, size: ShadcnSize) -> BadgeLook {
    badge_style(theme, variant, size).look()
}

impl IntoElement for Badge {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        use xgpui_luma::style::PartRead;
        let resolved = badge_style(&self.look, self.variant, self.size).resolve();
        let look = resolved.root();
        let typography = look.typography(12.0, 16.0);
        let mut root = div()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(look.number("gap")))
            .min_h(px(look.number("min_height")))
            .px(px(look.number("padding_x")))
            .py(px(look.number("padding_y")))
            .rounded(px(look.number("radius")))
            .bg(look.color("background"))
            .text_color(look.color("foreground"))
            .font_family(look.font_family_name())
            .font_weight(typography.weight)
            .text_size(px(typography.size))
            .line_height(px(typography.line_height))
            .when_some(look.painted("border"), |root, border| root.border_1().border_color(border));

        if let Some(icon) = self.icon.as_ref() {
            let rendered = render_badge_icon(icon, look.color("foreground"), look.number("icon_size"));
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
    use crate::tokens::ShadcnToken;

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

use gpui::{InteractiveElement, StatefulInteractiveElement, Styled, px};
use xgpui_luma::theme::{LumaTextScale, LumaTextStyle, LumaTypography};

use crate::context::with_active_look;
use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnStyle, ShadcnTextRole, ShadcnTextSize, ShadcnToken};

fn resolve_style_color(style: ShadcnStyle) -> Option<gpui::Hsla> {
    with_active_look(|look| {
        look.map(|look| {
            let mut color = look.color(style.token);
            if let Some(alpha) = style.opacity {
                color.a = alpha;
            }
            color
        })
    })
}

fn apply_text_style<S: Styled>(element: S, text_style: LumaTextStyle) -> S {
    element
        .text_size(px(text_style.size))
        .line_height(px(text_style.line_height))
        .font_weight(text_style.weight)
}

fn fallback_scale(size: ShadcnTextSize) -> LumaTextStyle {
    LumaTypography::default().text.scale(match size {
        ShadcnTextSize::Xs => LumaTextScale::Xs,
        ShadcnTextSize::Sm => LumaTextScale::Sm,
        ShadcnTextSize::Base => LumaTextScale::Md,
        ShadcnTextSize::Lg => LumaTextScale::Lg,
        ShadcnTextSize::Xl => LumaTextScale::Xl,
        ShadcnTextSize::TwoXl => LumaTextScale::TwoXl,
    })
}

fn apply_typography_scale<S: Styled>(element: S, size: ShadcnTextSize) -> S {
    with_active_look(|look| {
        if let Some(look) = look {
            apply_text_style(element, look.typography_scale(size))
        } else {
            apply_text_style(element, fallback_scale(size))
        }
    })
}

fn apply_typography_role<S: Styled>(element: S, role: ShadcnTextRole) -> S {
    with_active_look(|look| {
        if let Some(look) = look {
            apply_text_style(element, look.typography_role(role))
        } else {
            let fallback = match role {
                ShadcnTextRole::H1 => LumaTypography::default().text.role.h1,
                ShadcnTextRole::H2 => LumaTypography::default().text.role.h2,
                ShadcnTextRole::H3 => LumaTypography::default().text.role.h3,
                ShadcnTextRole::H4 => LumaTypography::default().text.role.h4,
                ShadcnTextRole::P => LumaTypography::default().text.role.p,
            };
            apply_text_style(element, fallback)
        }
    })
}

pub trait LumaTypographyExt: Styled + Sized {
    fn typography_style(self, style: LumaTextStyle) -> Self {
        apply_text_style(self, style)
    }

    fn text_h1(self) -> Self {
        apply_typography_role(self, ShadcnTextRole::H1)
    }

    fn text_h2(self) -> Self {
        apply_typography_role(self, ShadcnTextRole::H2)
    }

    fn text_h3(self) -> Self {
        apply_typography_role(self, ShadcnTextRole::H3)
    }

    fn text_h4(self) -> Self {
        apply_typography_role(self, ShadcnTextRole::H4)
    }

    fn text_p(self) -> Self {
        apply_typography_role(self, ShadcnTextRole::P)
    }

    fn typography_xs(self) -> Self {
        apply_typography_scale(self, ShadcnTextSize::Xs)
    }

    fn typography_sm(self) -> Self {
        apply_typography_scale(self, ShadcnTextSize::Sm)
    }

    fn typography_md(self) -> Self {
        apply_typography_scale(self, ShadcnTextSize::Base)
    }

    fn typography_lg(self) -> Self {
        apply_typography_scale(self, ShadcnTextSize::Lg)
    }

    fn typography_xl(self) -> Self {
        apply_typography_scale(self, ShadcnTextSize::Xl)
    }

    fn typography_2xl(self) -> Self {
        apply_typography_scale(self, ShadcnTextSize::TwoXl)
    }
}

/// Styling extensions that resolve shadcn tokens from the active look context.
pub trait ShadcnElementExt: Styled + Sized {
    fn bg_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        let style = style.into();
        if let Some(color) = resolve_style_color(style) {
            self.bg(color)
        } else {
            self
        }
    }

    fn text_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        let style = style.into();
        if let Some(color) = resolve_style_color(style) {
            self.text_color(color)
        } else {
            self
        }
    }

    fn border_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        let style = style.into();
        if let Some(color) = resolve_style_color(style) {
            self.border_color(color)
        } else {
            self
        }
    }

    fn hover_bg_cn(self, style: impl Into<ShadcnStyle>) -> Self
    where
        Self: InteractiveElement,
    {
        let style = style.into();
        self.hover(move |s| {
            if let Some(color) = resolve_style_color(style) {
                s.bg(color)
            } else {
                s
            }
        })
    }

    fn hover_text_cn(self, style: impl Into<ShadcnStyle>) -> Self
    where
        Self: InteractiveElement,
    {
        let style = style.into();
        self.hover(move |s| {
            if let Some(color) = resolve_style_color(style) {
                s.text_color(color)
            } else {
                s
            }
        })
    }

    fn active_bg_cn(self, style: impl Into<ShadcnStyle>) -> Self
    where
        Self: StatefulInteractiveElement,
    {
        let style = style.into();
        self.active(move |s| {
            if let Some(color) = resolve_style_color(style) {
                s.bg(color)
            } else {
                s
            }
        })
    }

    fn focus_border_cn(self, style: impl Into<ShadcnStyle>) -> Self
    where
        Self: InteractiveElement,
    {
        let style = style.into();
        self.focus(move |s| {
            if let Some(color) = resolve_style_color(style) {
                s.border_color(color)
            } else {
                s
            }
        })
    }

    /// Colors an existing border on focus using the active ring token.
    ///
    /// This low-level helper does not create an adorner or reserve space. Keep the
    /// border width constant in every state. SDK controls use `FocusAdorner` for
    /// independent focus paint and geometry.
    fn focus_ring_cn(self) -> Self
    where
        Self: InteractiveElement,
    {
        self.focus(|s| {
            if let Some(color) = resolve_style_color(ShadcnToken::Ring.into()) {
                s.border_color(color)
            } else {
                s
            }
        })
    }

    fn rounded_cn(self, role: ShadcnRadius) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                self.rounded(px(look.radius(role)))
            } else {
                self
            }
        })
    }

    /// Applies a catalog shadow token from the active look (`--shadow-*`).
    ///
    /// No-ops when no look is bound via [`crate::with_look`].
    fn shadow_cn(self, role: ShadcnShadow) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                self.shadow(look.shadow(role))
            } else {
                self
            }
        })
    }

    fn font_cn(self, role: ShadcnFont) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                self.font_family(look.font(role))
            } else {
                self
            }
        })
    }

    fn text_size_cn(self, size: ShadcnTextSize) -> Self {
        apply_typography_scale(self, size)
    }

    fn gap_cn(self, step: f32) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.gap(px(step * spacing))
            } else {
                self
            }
        })
    }

    fn p_cn(self, step: f32) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.p(px(step * spacing))
            } else {
                self
            }
        })
    }

    fn px_cn(self, step: f32) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.px(px(step * spacing))
            } else {
                self
            }
        })
    }

    fn py_cn(self, step: f32) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.py(px(step * spacing))
            } else {
                self
            }
        })
    }
}

impl<S: Styled> LumaTypographyExt for S {}
impl<S: Styled> ShadcnElementExt for S {}

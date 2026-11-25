use std::sync::Arc;

pub(crate) use xgpui_luma_look_radix::Look;
use xgpui_luma_look_radix::{Accent, Gray, SemanticRole};
use xgpui_luma::theme::{LumaChrome, LumaTextStyle, LumaTypography, ThemeMode};
use gpui::{Styled, px};

/// Monospace family bundled with the lab (`assets/fonts/ibm-plex-mono`, OFL).
/// Bundled so values render the same on every platform.
pub(crate) const MONO_FAMILY: &str = "IBM Plex Mono";
const IBM_PLEX_MONO: &[u8] = include_bytes!("../assets/fonts/ibm-plex-mono/IBMPlexMono-IBMPlexMonoRegular.ttf");

/// Register the lab's bundled fonts with GPUI. Call once at startup, before any window opens.
pub(crate) fn register_fonts(cx: &mut gpui::App) -> anyhow::Result<()> {
    cx.text_system().add_fonts(vec![std::borrow::Cow::Borrowed(IBM_PLEX_MONO)])
}

/// Radix orange accent and sand neutral palette used at startup.
pub(crate) fn default_look() -> Arc<Look> {
    let look = Look::built_in();
    // The theme's mono token is the lab's one monospace family; text roles read it from here.
    let mut fonts = look.typography().font;
    fonts.mono.family = MONO_FAMILY.into();
    look.set_fonts(fonts);
    // Named palettes only; `chrome()` keeps the light app background off-white
    // so native macOS traffic-light effects remain legible.
    look.set_palettes(Accent::Orange, Gray::Sand);
    look.set_mode(ThemeMode::Dark);
    Arc::new(look)
}

/// Typography and shell colors for this application, resolved from Radix roles.
pub(crate) trait ColorVizLookExt {
    fn chrome(&self) -> LumaChrome;
    fn interaction_colors(&self) -> InteractionColors;
    fn typography_scale(&self, size: TextSize) -> LumaTextStyle;
    fn scrollbar_template(&self) -> Arc<dyn xgpui_luma::controls::scrollbar::ScrollbarTemplate>;
}

/// Radix neutral state hierarchy, resolved from the shared custom palette.
#[derive(Clone, Copy)]
pub(crate) struct InteractionColors {
    pub selected: gpui::Hsla,
    pub hovered: gpui::Hsla,
    pub pressed: gpui::Hsla,
}
impl InteractionColors {
    pub fn background(self, selected: bool, state: xgpui_luma::theme::InteractionState) -> Option<gpui::Hsla> {
        if state.disabled {
            None
        } else if state.pressed {
            Some(self.pressed)
        } else if state.hovered || state.focused {
            Some(self.hovered)
        } else if selected {
            Some(self.selected)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum TextSize {
    Xs,
    Sm,
    Base,
}

impl ColorVizLookExt for Look {
    fn chrome(&self) -> LumaChrome {
        let color = |role| self.resolve_role(role).hsla();
        LumaChrome {
            // CustomColors.background seeds the scales; shell paint is an explicit app choice.
            app_background: if self.mode() == ThemeMode::Light {
                gpui::rgb(0xf0f0f0).into()
            } else {
                color(SemanticRole::Background)
            },
            content_background: color(SemanticRole::Background),
            panel_background: color(SemanticRole::Surface),
            border: color(SemanticRole::Border),
            title_text: color(SemanticRole::Foreground),
            body_text: color(SemanticRole::Foreground),
            muted_text: color(SemanticRole::MutedForeground),
        }
    }

    fn interaction_colors(&self) -> InteractionColors {
        let gray = |step| self.resolve_step(xgpui_luma_look_radix::ScaleFamily::Gray, step).hsla();
        InteractionColors { selected: gray(3), hovered: gray(4), pressed: gray(5) }
    }

    /// The lab's text sizes in pixels; the SDK has no text size scale.
    fn typography_scale(&self, size: TextSize) -> LumaTextStyle {
        let text = LumaTypography::default().text;
        match size {
            TextSize::Xs => text.caption,
            TextSize::Sm => LumaTextStyle { weight: text.body.weight, ..text.label },
            TextSize::Base => text.body,
        }
    }

    fn scrollbar_template(&self) -> Arc<dyn xgpui_luma::controls::scrollbar::ScrollbarTemplate> {
        Arc::new(xgpui_luma::controls::scrollbar::ThemedScrollbarTemplate::new(Arc::new(ColorVizScrollbarTheme(
            self.clone(),
        ))))
    }
}

pub(crate) trait TypographyExt: Styled + Sized {
    fn typography_style(self, style: LumaTextStyle) -> Self {
        self.text_size(px(style.size)).line_height(px(style.line_height)).font_weight(style.weight)
    }
}
impl<T: Styled> TypographyExt for T {}

pub(crate) fn icon_button(
    id: impl Into<gpui::SharedString>,
    icon: lucide_svg_static::Icon,
) -> xgpui_luma_look_radix::Button {
    xgpui_luma_look_radix::Button::new(id)
        .role(xgpui_luma::controls::button_family::ButtonFamilyRole::Icon)
        .icon(icon)
}

struct ColorVizScrollbarTheme(Look);
impl xgpui_luma::controls::scrollbar::ScrollbarTheme for ColorVizScrollbarTheme {
    fn resolve(
        &self,
        state: xgpui_luma::theme::InteractionState,
        orientation: xgpui_luma::controls::scrollbar::ScrollbarOrientation,
        style: xgpui_luma::controls::scrollbar::ScrollbarStyle,
    ) -> xgpui_luma::controls::scrollbar::ScrollbarLook {
        use xgpui_luma::controls::scrollbar::StyledScrollbarTheme;
        let mut resolved = StyledScrollbarTheme::sdk().resolve(state, orientation, style);
        resolved.track_background = gpui::transparent_black();
        resolved.thumb_background = self
            .0
            .resolve_step(
                xgpui_luma_look_radix::ScaleFamily::Gray,
                if state.pressed {
                    10
                } else if state.hovered {
                    9
                } else {
                    7
                },
            )
            .hsla();
        resolved
    }
    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.0.metrics()
    }
}

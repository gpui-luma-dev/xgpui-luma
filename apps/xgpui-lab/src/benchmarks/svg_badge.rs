//! Status badges in the mockup's colors: Radix `Badge` geometry and type, recolored here.
//! Radix tones are accent or gray only, so this modifies the badge element instead of the look.
use gpui::{Div, Hsla, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};
use xgpui_luma::theme::ThemeMode;
use xgpui_luma_look_radix::{self as radix, BadgeVariant, Look};

use crate::theme::ColorVizLookExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Status {
    /// Pending, Ready, Stopped, the build profile.
    Neutral,
    /// Running, or partly measured.
    Running,
    Done,
    Failed,
}

impl Status {
    /// Text and outline colors, `None` for the look's muted text and border.
    fn colors(self, mode: ThemeMode) -> Option<(u32, u32)> {
        let dark = mode == ThemeMode::Dark;
        match self {
            Self::Neutral => None,
            Self::Running => Some(if dark {
                (0x9cc8ff, 0x24466e)
            } else {
                (0x1f63b8, 0xa9c8ef)
            }),
            Self::Done => Some(if dark {
                (0x9fe3a1, 0x2f5a33)
            } else {
                (0x2b7a3b, 0x9fd5a8)
            }),
            Self::Failed => Some(if dark {
                (0xff9592, 0x6e2a2c)
            } else {
                (0xc4322f, 0xf0b2b0)
            }),
        }
    }
}

fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// A pill badge with a status outline. Size, padding and font come from Radix `Badge`.
pub(super) fn status_badge(look: &Look, label: impl Into<SharedString>, status: Status) -> Div {
    let chrome = look.chrome();
    let (text, outline) = match status.colors(look.mode()) {
        Some((text, outline)) => (color(text), color(outline)),
        None => (chrome.muted_text, chrome.border),
    };
    radix::Badge::new(look, label)
        .variant(BadgeVariant::Soft)
        .pill()
        .into_element()
        .bg(gpui::transparent_black())
        .text_color(text)
        .child(div().absolute().inset_0().rounded(px(999.)).border_1().border_color(outline))
}

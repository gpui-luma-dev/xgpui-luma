//! Every size, gap and type role a lab screen uses. Defaults are Color Spaces' values.
use gpui::{Div, FontWeight, ParentElement, Pixels, Rems, SharedString, Styled, div, px, rems};
use xgpui_luma::{controls::textfield::TextFieldLook, theme::LumaTextStyle};
use xgpui_luma_look_radix::Look;

use crate::theme::{ColorVizLookExt, TextSize, TypographyExt};

/// Layout metrics shared by every lab screen.
///
/// Spacing is in rems so it matches GPUI's `p_6` / `gap_5` shorthands exactly.
#[derive(Clone, Copy)]
pub(crate) struct LabMetrics {
    pub max_width: Pixels,
    pub page_padding: Rems,
    /// Between crumb, header and content.
    pub part_gap: Rems,
    /// Between title and lede.
    pub header_gap: Rems,
    /// Between content sections, cards and panels.
    pub section_gap: Rems,
    /// Between items in the control row.
    pub control_gap: Rems,
    pub card_min_width: Pixels,
    pub card_swatch: Pixels,
    /// Small data-color square before a card name.
    pub card_mark: Pixels,
    /// Minimum width of a hero visual that sits beside a legend.
    pub hero_min_width: Pixels,
    pub legend_width: Pixels,
    /// Width of the mark column in a legend row.
    pub legend_mark_width: Pixels,
    pub inline_slider_width: Pixels,
    pub param_min_width: Pixels,
    pub preview_height: Pixels,
    /// Diagram axis tick labels (mono, muted).
    pub axis_tick: Pixels,
    pub wide_panel: PanelWidth,
    pub narrow_panel: PanelWidth,
}

/// Flex sizing of one side of the two-panel row.
#[derive(Clone, Copy)]
pub(crate) struct PanelWidth {
    pub grow: f32,
    pub basis: Pixels,
    pub min: Pixels,
}

impl LabMetrics {
    pub const STANDARD: Self = Self {
        max_width: px(1240.),
        page_padding: rems(1.5),
        part_gap: rems(1.25),
        header_gap: rems(0.5),
        section_gap: rems(1.),
        control_gap: rems(0.75),
        card_min_width: px(200.),
        card_swatch: px(28.),
        card_mark: px(6.),
        hero_min_width: px(400.),
        legend_width: px(220.),
        legend_mark_width: px(24.),
        inline_slider_width: px(200.),
        param_min_width: px(100.),
        preview_height: px(80.),
        axis_tick: px(10.),
        wide_panel: PanelWidth { grow: 2., basis: px(440.), min: px(240.) },
        narrow_panel: PanelWidth { grow: 1., basis: px(265.), min: px(225.) },
    };
}

/// The one monospace family for values. Screens never name a font family themselves.
/// It must be a real installed family: GPUI passes names to the OS as-is, and the
/// generic CSS name `"monospace"` silently falls back to the UI font.
#[cfg(target_os = "macos")]
pub(crate) const MONO_FAMILY: &str = "Menlo";
#[cfg(target_os = "windows")]
pub(crate) const MONO_FAMILY: &str = "Consolas";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) const MONO_FAMILY: &str = "DejaVu Sans Mono";

/// Text roles. A screen picks a role; the look decides size, weight and color.
#[derive(Clone, Copy)]
pub(crate) enum TextRole {
    Crumb,
    Title,
    Lede,
    /// Panel title.
    Section,
    /// Muted small text: live state, notices, notes, subtitles.
    Caption,
    /// Card name.
    Name,
    /// Keys and small labels (UI face, muted): `R`, `x`, `Clip`, `Source`.
    Label,
    /// Small words (UI face): legend names.
    Detail,
    /// Measured or edited values (mono): channel values, coordinates, hex, percentages.
    Value,
    /// Credits and provenance.
    Fine,
}

/// Mono value size. Menlo runs larger and heavier than the UI face at the same
/// size, so values sit a half step below the 12 px detail text they pair with.
pub(crate) const VALUE_SIZE: f32 = 11.5;

fn detail_style(look: &Look) -> LumaTextStyle {
    LumaTextStyle { size: 12., ..look.typography_scale(TextSize::Sm) }
}

impl TextRole {
    /// Apply this role's typography and color.
    pub fn apply<E: Styled>(self, look: &Look, element: E) -> E {
        let muted = look.chrome().muted_text;
        match self {
            Self::Crumb | Self::Lede | Self::Caption => element.text_sm().text_color(muted),
            Self::Title => element.text_3xl().font_weight(FontWeight::SEMIBOLD),
            Self::Section => element.text_lg().font_weight(FontWeight::SEMIBOLD),
            Self::Name => {
                element.typography_style(look.typography_scale(TextSize::Base)).font_weight(FontWeight::SEMIBOLD)
            }
            Self::Label => element.text_color(muted).typography_style(detail_style(look)),
            Self::Detail => element.typography_style(detail_style(look)),
            Self::Value => element
                .typography_style(LumaTextStyle { size: VALUE_SIZE, ..detail_style(look) })
                .font_family(MONO_FAMILY),
            Self::Fine => element.text_xs().text_color(muted),
        }
    }
}

/// A text element in the given role.
pub(crate) fn text(look: &Look, role: TextRole, content: impl Into<SharedString>) -> Div {
    role.apply(look, div()).child(content.into())
}

/// Key/value pairs: keys in `Label` (UI, muted), values in `Value` (mono). An empty key shows the value alone.
pub(crate) fn readout<K: Into<SharedString>, V: Into<SharedString>>(
    look: &Look,
    pairs: impl IntoIterator<Item = (K, V)>,
) -> Div {
    div().flex().flex_wrap().items_baseline().gap_2().children(pairs.into_iter().map(|(key, value)| {
        let key: SharedString = key.into();
        div()
            .flex()
            .items_baseline()
            .gap_1()
            .children((!key.is_empty()).then(|| text(look, TextRole::Label, key)))
            .child(text(look, TextRole::Value, value))
    }))
}

/// Text-field look for editing a value: the same mono 12 px as `TextRole::Value`.
pub(crate) fn value_field_look(mut field: TextFieldLook) -> TextFieldLook {
    field.typography = LumaTextStyle { size: VALUE_SIZE, line_height: 16., ..field.typography };
    field.font_family = MONO_FAMILY.into();
    field
}

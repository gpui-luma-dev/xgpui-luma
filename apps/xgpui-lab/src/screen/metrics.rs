//! Every size, gap and type role a lab screen uses. Defaults are Color Spaces' values.
use gpui::{Div, ParentElement, Pixels, Rems, SharedString, Styled, div, px, rems};
use xgpui_luma::{
    controls::textfield::TextFieldLook,
    theme::{LumaTextStyle, tokens::TextTokens},
};
use xgpui_luma_look_radix::Look;

use crate::theme::{ColorVizLookExt, TypographyExt};

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
    /// Status-card line icon (drawn plain, without a box).
    pub card_icon: Pixels,
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
    /// Photo-card variant (Italy): card width and its 3:2 thumbnail.
    pub photo_card_width: Pixels,
    pub photo_thumbnail: gpui::Size<Pixels>,
    /// Label column and minimum width of a form group.
    pub form_label_width: Pixels,
    pub form_group_min_width: Pixels,
    /// Key column of a properties list.
    pub property_key_width: Pixels,
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
        card_icon: px(32.),
        card_mark: px(6.),
        hero_min_width: px(400.),
        legend_width: px(220.),
        legend_mark_width: px(24.),
        inline_slider_width: px(200.),
        param_min_width: px(100.),
        preview_height: px(80.),
        axis_tick: px(10.),
        photo_card_width: px(200.),
        photo_thumbnail: gpui::Size { width: px(174.), height: px(116.) },
        property_key_width: px(140.),
        form_label_width: px(104.),
        form_group_min_width: px(200.),
        wide_panel: PanelWidth { grow: 2., basis: px(440.), min: px(240.) },
        narrow_panel: PanelWidth { grow: 1., basis: px(265.), min: px(225.) },
    };
}

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
    /// Parameter labels above a field: the panel subtitle's size, in foreground.
    Field,
    /// Keys and small labels (UI face, muted): `R`, `x`, `Clip`, `Source`.
    Label,
    /// Small words (UI face): legend names.
    Detail,
    /// Measured or edited values (mono): channel values, coordinates, hex, percentages.
    Value,
    /// Credits and provenance.
    Fine,
}

/// The look's text tokens: `role.h1`–`h4` and `p`, plus `body`, `label`, `caption`, `title` and `code`.
fn tokens(look: &Look) -> TextTokens {
    look.common_stylesheet().tokens.typography.text
}

/// The look's monospace family (`font.mono`), used for every value.
pub(crate) fn mono_family(look: &Look) -> SharedString {
    look.common_stylesheet().tokens.typography.font.mono.family.clone().into()
}

impl TextRole {
    /// The look token this role uses. Sizes, line heights and weights all come from the look.
    pub fn style(self, look: &Look) -> LumaTextStyle {
        let text = tokens(look);
        match self {
            Self::Title => text.role.h2,
            Self::Section => text.role.h3,
            Self::Name => text.role.h4,
            Self::Crumb | Self::Lede | Self::Caption | Self::Field => text.role.p,
            Self::Label | Self::Detail => text.label,
            Self::Value => text.code,
            Self::Fine => text.caption,
        }
    }

    /// Apply this role's typography and color.
    pub fn apply<E: Styled>(self, look: &Look, element: E) -> E {
        let element = element.typography_style(self.style(look));
        match self {
            Self::Crumb | Self::Lede | Self::Caption | Self::Label | Self::Fine => {
                element.text_color(look.chrome().muted_text)
            }
            Self::Value => element.font_family(mono_family(look)),
            Self::Title | Self::Section | Self::Name | Self::Field | Self::Detail => element,
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

/// Text-field look for editing a value: the same mono `code` style as `TextRole::Value`.
pub(crate) fn value_field_look(look: &Look) -> impl Fn(TextFieldLook) -> TextFieldLook + Send + Sync + 'static {
    let style = TextRole::Value.style(look);
    let family = mono_family(look);
    move |mut field| {
        field.typography = LumaTextStyle { weight: field.typography.weight, ..style };
        field.font_family = family.clone();
        field
    }
}

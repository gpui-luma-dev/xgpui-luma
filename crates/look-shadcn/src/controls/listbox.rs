//! List box — input surface tokens; accent row hover and keyboard focus.
//!
//! | Part           | Token              |
//! |----------------|--------------------|
//! | List bg        | `background`       |
//! | Disabled list  | `muted`            |
//! | Border         | `input`            |
//! | Divider        | `border`           |
//! | Row hover      | `accent` (layer)   |
//! | Focused row    | `accent`           |
//! | Disabled label | `muted-foreground` |

use crate::stylesheet::config::{EnabledColorRule, find_enabled_color_rule, matches_optional_bool, matches_optional_layer};
use serde::Deserialize;

use gpui::{Div, ElementId, Stateful};
use xgpui_luma::controls::listbox::ListBoxItemState;
use xgpui_luma::theme::InteractionLayer;

use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{StylesheetConfig, find_listbox_row_color_rule, resolve_listbox_row_color_rule};

impl crate::ShadcnLook {
    /// Styled, content-free row surface. Attach SDK `ListBoxBinding` for input;
    /// the host supplies arbitrary content and all layout dimensions.
    /// The listbox chrome from `.lss` (`style/listbox.lss`).
    pub fn styled_listbox(&self) -> xgpui_luma::controls::listbox::StyledListBox {
        let look = self.clone();
        xgpui_luma::controls::listbox::StyledListBox::new(xgpui_luma::style::StyleBinding::new(
            "listbox",
            xgpui_luma::style::ThemeSource::Live(std::sync::Arc::new(move || look.style_theme())),
        ))
    }

    pub fn listbox_row(&self, id: impl Into<ElementId>, state: ListBoxItemState, focus_visible: bool) -> Stateful<Div> {
        self.styled_listbox().paint_row(id, state, focus_visible)
    }
}

#[derive(Clone, Debug)]
pub struct ListBoxListColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
}

impl ListBoxListColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            border: ResolvedColor::fallback_foreground(),
            divider: ResolvedColor::fallback_foreground(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ListBoxRowColorTable {
    pub label_color: ResolvedColor,
    pub background: ResolvedColor,
}

impl ListBoxRowColorTable {
    pub fn fallback() -> Self {
        Self { label_color: ResolvedColor::fallback_foreground(), background: ResolvedColor::transparent() }
    }
}

pub fn resolve_listbox_row_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    focused: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ListBoxRowColorTable> {
    resolve_listbox_row_colors_with_stylesheet(resolver, resolver.stylesheet(), disabled, focused, layer)
}

pub fn resolve_listbox_row_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    focused: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ListBoxRowColorTable> {
    let rule = find_listbox_row_color_rule(stylesheet, disabled, focused, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching listbox row color rule"))?;
    let colors = resolve_listbox_row_color_rule(resolver, rule, layer)?;
    Ok(ListBoxRowColorTable { label_color: colors.label_color, background: colors.background })
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ListboxStylesheet {
    #[serde(default)]
    pub list: ListboxListStylesheet,
    #[serde(default)]
    pub row: ListboxRowStylesheet,
}

impl ListboxStylesheet {
    /// Native bundled list and row rules in priority order.
    pub fn native_default() -> Self {
        Self {
            list: ListboxListStylesheet {
                color_rules: [(true, "background"), (false, "muted")]
                    .into_iter()
                    .map(|(enabled, background)| ListboxListColorRule {
                        enabled: Some(enabled),
                        background: background.into(),
                        border: "input".into(),
                        divider: "border".into(),
                    })
                    .collect(),
            },
            row: ListboxRowStylesheet {
                color_rules: [
                    (true, None, None, "muted-foreground", "transparent"),
                    (false, Some(true), Some("default"), "foreground", "accent"),
                    (false, Some(false), Some("default"), "foreground", "transparent"),
                    (false, None, Some("hover"), "foreground", "first_layer(accent)"),
                    (false, None, Some("pressed"), "foreground", "first_layer(accent)"),
                    (false, None, Some("disabled"), "foreground", "transparent"),
                ]
                .into_iter()
                .map(|(disabled, focused, layer, label, background)| ListboxRowColorRule {
                    disabled: Some(disabled),
                    focused,
                    layer: layer.map(Into::into),
                    label_color: label.into(),
                    background: background.into(),
                })
                .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ListboxListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ListboxListColorRule>,
}

impl ListboxListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&ListboxListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ListboxListColorRule {
    pub enabled: Option<bool>,
    pub background: String,
    pub border: String,
    pub divider: String,
}

impl EnabledColorRule for ListboxListColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ListboxRowStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ListboxRowColorRule>,
}

impl ListboxRowStylesheet {
    pub fn find_color_rule(
        &self,
        disabled: bool,
        focused: bool,
        layer: InteractionLayer,
    ) -> Option<&ListboxRowColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_bool(rule.focused, focused) && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ListboxRowColorRule {
    pub disabled: Option<bool>,
    pub focused: Option<bool>,
    pub layer: Option<String>,
    pub label_color: String,
    pub background: String,
}

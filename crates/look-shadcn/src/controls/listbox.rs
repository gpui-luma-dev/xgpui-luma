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

use gpui::{Div, ElementId, Hsla, Stateful, div, px, prelude::*};
use xgpui_luma::controls::listbox::ListBoxItemState;
use xgpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, find_listbox_list_color_rule, find_listbox_row_color_rule, resolve_listbox_list_color_rule,
    resolve_listbox_row_color_rule,
};

/// Look-owned styling for host-composed list surfaces; not an SDK theme.
#[derive(Clone, Debug)]
pub struct ListBoxSurfacePalette {
    pub background: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Debug)]
pub struct ListBoxRowPalette {
    pub background: Hsla,
    pub label_color: Hsla,
    pub label_typography: LumaTextStyle,
}

impl crate::ShadcnLook {
    /// Styled, content-free row surface. Attach SDK `ListBoxBinding` for input;
    /// the host supplies arbitrary content and all layout dimensions.
    pub fn listbox_row(&self, id: impl Into<ElementId>, state: ListBoxItemState, focus_visible: bool) -> Stateful<Div> {
        let mode = self.mode_tokens();
        let base = listbox_row_palette(
            &mode,
            state.selected,
            InteractionState { disabled: !state.enabled, ..Default::default() },
            ControlSize::Md,
        );
        let hover = listbox_row_palette(
            &mode,
            state.selected,
            InteractionState { hovered: true, ..Default::default() },
            ControlSize::Md,
        );
        let pressed = listbox_row_palette(
            &mode,
            state.selected,
            InteractionState { pressed: true, ..Default::default() },
            ControlSize::Md,
        );
        let border = if state.active && focus_visible {
            crate::focus::focus_ring_or_fallback(&mode.catalog)
        } else {
            gpui::transparent_black()
        };
        div()
            .id(id)
            .bg(base.background)
            .text_color(base.label_color)
            .text_size(px(base.label_typography.size))
            .line_height(px(base.label_typography.line_height))
            .font_weight(base.label_typography.weight)
            .border_1()
            .border_color(border)
            .when(state.enabled, |row| {
                row.cursor_pointer()
                    .hover(|style| style.bg(hover.background).text_color(hover.label_color))
                    .active(|style| style.bg(pressed.background).text_color(pressed.label_color))
            })
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

pub fn resolve_listbox_list_colors(
    resolver: &LookResolver<'_>,
    enabled: bool,
) -> anyhow::Result<ListBoxListColorTable> {
    resolve_listbox_list_colors_with_stylesheet(resolver, resolver.stylesheet(), enabled)
}

pub fn resolve_listbox_list_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<ListBoxListColorTable> {
    let rule = find_listbox_list_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching listbox list color rule"))?;
    let colors = resolve_listbox_list_color_rule(resolver, rule)?;
    Ok(ListBoxListColorTable { background: colors.background, border: colors.border, divider: colors.divider })
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

/// Color palette only; the host owns all surface geometry.
pub fn listbox_surface_palette(mode: &ShadcnModeTokens, enabled: bool) -> ListBoxSurfacePalette {
    let ctx = LookContext::new(mode, mode.theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "listbox_list").with_stylesheet(mode.stylesheet());
    let colors = resolve_listbox_list_colors(&resolver, enabled).unwrap_or_else(|_| ListBoxListColorTable::fallback());
    ListBoxSurfacePalette { background: colors.background.hsla(), border: colors.border.hsla() }
}

pub fn listbox_row_palette(
    mode: &ShadcnModeTokens,
    selected: bool,
    mut state: InteractionState,
    size: ControlSize,
) -> ListBoxRowPalette {
    state.focused |= selected;
    let ctx = LookContext::new(mode, mode.theme_mode, state);
    let typography = ctx.typography();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "listbox_row").with_stylesheet(mode.stylesheet());
    let colors = resolve_listbox_row_colors(&resolver, state.disabled, state.focused, state.layer())
        .unwrap_or_else(|_| ListBoxRowColorTable::fallback());

    let mut label_typography = typography.text.label;
    super::apply_button_metrics_typography(&mut label_typography, mode, size);

    ListBoxRowPalette { background: colors.background.hsla(), label_color: colors.label_color.hsla(), label_typography }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::{ControlSize, InteractionLayer, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::provenance::LookResolver;
    use super::{listbox_surface_palette, listbox_row_palette, resolve_listbox_row_colors};

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn listbox_uses_input_border_and_accent_hover() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let list = listbox_surface_palette(&mode, true);
        let row = listbox_row_palette(
            &mode,
            false,
            xgpui_luma::theme::InteractionState { hovered: true, ..Default::default() },
            ControlSize::Md,
        );

        assert_eq!(list.border, catalog.color("input").expect("input"));
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let expected = resolve_listbox_row_colors(&resolver, false, false, InteractionLayer::Hovered)
            .expect("row colors")
            .background
            .hsla();
        assert_eq!(row.background, expected);
    }
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

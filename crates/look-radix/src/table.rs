//! Radix table shell, header, row, and drag styling over the SDK table engine.
use std::sync::Arc;
use gpui::{App, AnyElement, SharedString, Stateful, Div, Window, Styled};
use xgpui_luma::controls::table::*;
use xgpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ListRowScale, MetricTokens, StandardBoxScale};
use crate::{Look, ScaleFamily, SemanticRole, Tone};

struct TableThemeAdapter {
    look: Look,
}
impl TableTheme for TableThemeAdapter {
    fn resolve_look(&self, enabled: bool, focused: bool, size: ControlSize) -> TableLook {
        let mut resolved =
            DefaultTableTheme::new(crate::navigation::geometry_tokens(&self.look)).resolve_look(enabled, focused, size);
        resolved.background = self.look.resolve_role(SemanticRole::Surface).hsla();
        resolved.border = self
            .look
            .resolve_role(if enabled && focused {
                SemanticRole::Focus
            } else {
                SemanticRole::Border
            })
            .hsla();
        resolved.header_background = self.look.resolve_step(ScaleFamily::Gray, 3).hsla();
        resolved.header_label_color = self
            .look
            .resolve_role(if enabled {
                SemanticRole::Foreground
            } else {
                SemanticRole::MutedForeground
            })
            .hsla();
        resolved
    }
    fn resolve_row(&self, selected: bool, state: InteractionState, size: ControlSize) -> TableRowPalette {
        let mut resolved =
            DefaultTableTheme::new(crate::navigation::geometry_tokens(&self.look)).resolve_row(selected, state, size);
        let gray = |step| self.look.resolve_step(ScaleFamily::Gray, step).hsla();
        let accent = |step| Tone::Accent.step(&self.look, step);
        resolved.background = match state.layer() {
            InteractionLayer::Disabled => gpui::transparent_black(),
            InteractionLayer::Pressed => {
                if selected {
                    accent(5)
                } else {
                    gray(4)
                }
            }
            InteractionLayer::Hovered => {
                if selected {
                    accent(4)
                } else {
                    gray(3)
                }
            }
            InteractionLayer::Default => {
                if selected {
                    accent(3)
                } else if state.focused {
                    accent(2)
                } else {
                    gpui::transparent_black()
                }
            }
        };
        resolved.label_color = if state.disabled {
            self.look.resolve_role(SemanticRole::MutedForeground).hsla()
        } else if selected {
            accent(11)
        } else {
            self.look.resolve_role(SemanticRole::Foreground).hsla()
        };
        resolved.divider = self.look.resolve_role(SemanticRole::Border).hsla();
        let field_theme = crate::textfield_theme(&self.look);
        let scale = StandardBoxScale::compute(size, &self.look.metrics(), 1.0);
        resolved.label_typography =
            field_theme.resolve_look(Default::default(), !state.disabled, size, &scale).typography;
        resolved
    }
    fn resolve_row_look(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
        scale: &ListRowScale,
    ) -> TableRowLook {
        TableRowLook::from_palette(
            &self.resolve_row(selected, state, size),
            scale,
            &self.look.common_stylesheet(),
            size,
        )
    }
    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
    fn resolve_drag(&self, size: ControlSize) -> TableDragLook {
        TableDragLook {
            geometry: self
                .look
                .common_stylesheet()
                .table
                .with_sdk_defaults()
                .resolve_geometry(size.stylesheet_key(), Default::default()),
            background: self.look.resolve_role(SemanticRole::Surface).hsla(),
            foreground: self.look.resolve_role(SemanticRole::Foreground).hsla(),
            valid_marker: self.look.resolve_role(SemanticRole::Focus).hsla(),
            invalid_marker: self.look.resolve_role(SemanticRole::Destructive).hsla(),
            radius: self.look.metrics().radius(size),
        }
    }
}
struct TableTemplateAdapter {
    look: Look,
}
impl TableTemplate for TableTemplateAdapter {
    fn render(
        &self,
        model: &TableRenderModel<'_>,
        header: Option<AnyElement>,
        body: AnyElement,
        _: &mut Window,
        _: &mut App,
    ) -> Stateful<Div> {
        DefaultTableShellTemplate::paint_shell(model, header, body)
            .font_family(crate::typography::font_family(&self.look))
    }
}
/// Live Radix header, row, focus, disabled, and drag styling with SDK geometry.
/// Retains the supplied Look; notify affected controls and owners after changing it.
pub fn table_theme(look: &Look) -> Arc<dyn TableTheme> {
    Arc::new(TableThemeAdapter { look: look.clone() })
}
/// SDK shell renderer with Radix typography. Pair with [`table_theme`] when composing manually.
pub fn table_template(look: &Look) -> Arc<dyn TableTemplate> {
    Arc::new(TableTemplateAdapter { look: look.clone() })
}
/// Themed SDK table builder, preserving callbacks, selection, virtualization, and column APIs.
/// Row keys, reordering, and owner-managed selection remain SDK control APIs.
///
/// ```no_run
/// use gpui::Context;
/// use xgpui_luma::controls::table::TableLabel;
/// use xgpui_luma_look_radix::{Look, table_builder, paging_table_builder};
/// fn tables<M: 'static>(look: &Look, cx: &mut Context<M>) {
///     let table = table_builder(look, "records").items([TableLabel::new("First")])
///         .extended().visible_rows(4).spawn(cx);
///     let paging = paging_table_builder(table_builder(look, "paged").items([TableLabel::new("First")]), look, "pager")
///         .spawn(cx);
/// }
/// ```
pub fn table_builder<T: 'static>(look: &Look, id: impl Into<SharedString>) -> TableBuilder<T> {
    TableBuilder::new_typed(id)
        .motion_defaults(look.motion_defaults().table)
        .theme(table_theme(look))
        .template(table_template(look))
}
/// Compose a configured SDK table builder with the Radix Pager.
/// Configure columns, selection, and policies on the table builder before composition.
pub fn paging_table_builder<T: 'static>(
    table: TableBuilder<T>,
    look: &Look,
    pager_id: impl Into<SharedString>,
) -> PagingTableBuilder<T> {
    PagingTableBuilder::new(table, crate::pager_builder(look, pager_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Accent, Gray};
    use xgpui_luma::theme::ThemeMode;
    #[test]
    fn table_states_follow_live_look_and_disabled_precedence() {
        let look = Look::built_in();
        let theme = table_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for accent in [Accent::Blue, Accent::Lime] {
                look.set_palettes(accent, Gray::Auto);
                for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                    for selected in [false, true] {
                        for bits in 0..16 {
                            let state = InteractionState {
                                hovered: bits & 1 != 0,
                                pressed: bits & 2 != 0,
                                focused: bits & 4 != 0,
                                disabled: bits & 8 != 0,
                                ..Default::default()
                            };
                            let row = theme.resolve_row(selected, state, size);
                            if state.disabled {
                                assert_eq!(row.background, gpui::transparent_black());
                                assert_eq!(row.label_color, look.resolve_role(SemanticRole::MutedForeground).hsla());
                            } else if selected {
                                let step = if state.pressed {
                                    5
                                } else if state.hovered {
                                    4
                                } else {
                                    3
                                };
                                assert_eq!(row.background, Tone::Accent.step(&look, step));
                                assert_eq!(row.label_color, Tone::Accent.step(&look, 11));
                            }
                            let shell = theme.resolve_look(!state.disabled, state.focused, size);
                            assert_eq!(
                                shell.border,
                                look.resolve_role(if !state.disabled && state.focused {
                                    SemanticRole::Focus
                                } else {
                                    SemanticRole::Border
                                })
                                .hsla()
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn table_forks_keep_geometry_and_palettes_local() {
        let base = Look::built_in();
        let fork = base.fork();
        let theme = table_theme(&fork);
        let original = table_theme(&base).resolve_look(true, false, ControlSize::Md);
        let mut sheet = fork.common_stylesheet();
        sheet.table.geometry.column_padding_x = Some(23.0);
        fork.set_common_stylesheet(sheet);
        fork.set_mode(ThemeMode::Dark);
        let changed = theme.resolve_look(true, false, ControlSize::Md);
        assert_eq!(changed.column_padding_x, 23.0);
        assert_ne!(changed.background, original.background);
        assert_eq!(
            table_theme(&base).resolve_look(true, false, ControlSize::Md).column_padding_x,
            original.column_padding_x
        );
        let drag = theme.resolve_drag(ControlSize::Md);
        assert_eq!(drag.valid_marker, fork.resolve_role(SemanticRole::Focus).hsla());
        assert_eq!(drag.invalid_marker, fork.resolve_role(SemanticRole::Destructive).hsla());
    }
}

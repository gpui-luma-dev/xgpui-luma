//! Radix table shell, header, row, and drag styling over the SDK table engine.
use std::sync::Arc;
use gpui::{App, AnyElement, SharedString, Stateful, Div, Window, Styled};
use xgpui_luma::controls::table::*;
use xgpui_luma::theme::MetricTokens;
use crate::Look;

/// The Radix table: frame, header and rows from `.lss` (`style/table.lss`); the drag preview is the look's
/// `.lss` drag preview.
struct TableThemeAdapter {
    look: Look,
    styled: StyledTable,
}
impl TableTheme for TableThemeAdapter {
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        self.styled.binding()
    }
    fn metrics(&self) -> MetricTokens {
        self.look.metrics_for(self.look.size())
    }
    fn resolve_drag(&self) -> TableDragLook {
        table_drag_look(crate::navigation::drag_look(&self.look, self.look.size()), self.styled.drag_geometry())
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
    let size = look.size();
    let binding = xgpui_luma::style::StyleBinding::new("table", crate::textfield::live_source(look));
    let binding = if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(TableThemeAdapter { look: look.clone(), styled: StyledTable::new(binding) })
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
    use crate::{SemanticRole, Tone};
    use crate::control_group::assert_close;
    use super::*;
    use crate::{Accent, Gray};
    use xgpui_luma::style::PartRead;
    use xgpui_luma::style::lss::PartId;
    use xgpui_luma::theme::{InteractionState, ThemeMode};

    /// The frame's resolved background in `theme`, enabled and unfocused.
    fn frame_background(theme: &Arc<dyn TableTheme>) -> gpui::Hsla {
        StyledTable::new(theme.style()).resolve(true, false).root().color("background")
    }

    #[test]
    fn table_states_follow_live_look_and_disabled_precedence() {
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for accent in [Accent::Blue, Accent::Lime] {
                look.set_palettes(accent, Gray::Auto);
                for size in [crate::RadixSize::One, crate::RadixSize::Two, crate::RadixSize::Three] {
                    let theme = StyledTable::new(table_theme(&look.with_size(size)).style());
                    for selected in [false, true] {
                        for bits in 0..16 {
                            let state = InteractionState {
                                hovered: bits & 1 != 0,
                                pressed: bits & 2 != 0,
                                focused: bits & 4 != 0,
                                disabled: bits & 8 != 0,
                                ..Default::default()
                            };
                            let row = theme.resolve_row(selected, state);
                            let row = row.part(PartId::new("row"));
                            if state.disabled {
                                assert_eq!(row.color("background").a, 0.0);
                                assert_close(
                                    row.color("foreground"),
                                    look.resolve_role(SemanticRole::MutedForeground).hsla(),
                                );
                            } else if selected {
                                let step = if state.pressed {
                                    5
                                } else if state.hovered {
                                    4
                                } else {
                                    3
                                };
                                assert_close(row.color("background"), Tone::Accent.step(&look, step));
                                assert_close(row.color("foreground"), Tone::Accent.step(&look, 11));
                            }
                            let shell = theme.resolve(!state.disabled, state.focused);
                            assert_close(
                                shell.root().color("border"),
                                look.resolve_role(if !state.disabled && state.focused {
                                    SemanticRole::Focus
                                } else {
                                    SemanticRole::Border
                                })
                                .hsla(),
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn table_forks_keep_palettes_local() {
        let base = Look::built_in();
        let fork = base.fork();
        let theme = table_theme(&fork);
        let original = frame_background(&table_theme(&base));
        fork.set_mode(ThemeMode::Dark);
        assert_ne!(frame_background(&theme), original);
        assert_eq!(frame_background(&table_theme(&base)), original);
        let drag = theme.resolve_drag();
        assert_close(drag.valid_marker, fork.resolve_role(SemanticRole::Focus).hsla());
        assert_close(drag.invalid_marker, fork.resolve_role(SemanticRole::Destructive).hsla());
    }
}

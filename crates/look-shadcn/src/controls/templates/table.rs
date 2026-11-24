use std::sync::Arc;

use xgpui_luma::controls::table::{StyledTable, TableTheme, table_drag_look, table_template_with_theme};
use xgpui_luma::theme::{InteractionState};

use crate::look::ShadcnLook;

/// The Shadcn table: frame, header and rows from `.lss` (`style/table.lss`); the drag preview is the look's
/// `.lss` drag preview.
struct ShadcnTableTheme {
    theme: ShadcnLook,
    styled: StyledTable,
}

impl TableTheme for ShadcnTableTheme {
    fn resolve_drag(&self) -> xgpui_luma::controls::table::TableDragLook {
        table_drag_look(super::drag::drag_look(&self.theme), self.styled.drag_geometry())
    }

    fn resolve_look(&self, enabled: bool, focused: bool) -> xgpui_luma::controls::table::TableLook {
        self.styled.look(enabled, focused)
    }

    fn resolve_row(&self, selected: bool, state: InteractionState) -> xgpui_luma::controls::table::TableRowPalette {
        self.styled.row(selected, state)
    }

    fn resolve_row_look(
        &self,
        selected: bool,
        state: InteractionState,
        scale: &xgpui_luma::theme::ListRowScale,
    ) -> xgpui_luma::controls::table::TableRowLook {
        self.styled.row_look(selected, state, scale)
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics_for(self.theme.size())
    }
}

pub fn table_theme(theme: ShadcnLook) -> Arc<dyn TableTheme> {
    let size = theme.size();
    let style = theme.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "table",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || style.style_theme())),
    );
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(ShadcnTableTheme { theme, styled: StyledTable::new(binding) })
}

pub fn table_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::table::TableTemplate> {
    table_template_with_theme(table_theme(theme))
}

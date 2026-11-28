use std::sync::Arc;

use gpui::SharedString;
use xgpui_luma::style::PartRead;
use xgpui_luma::controls::textarea::TextAreaState;
use xgpui_luma::theme::{StandardBoxScale};
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};

use super::model::EventLogViewBuilder;
use super::theme::{EventLogTheme, compose_event_log_look};

struct ShadcnEventLogTheme {
    look: Arc<ShadcnLook>,
}

impl EventLogTheme for ShadcnEventLogTheme {
    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.look.mode_tokens().metrics
    }

    fn resolve_look(&self, rows: usize, scale: &StandardBoxScale, focused: bool) -> super::theme::EventLogLook {
        let textarea_style = self
            .look
            .textarea_theme()
            .resolve_style(TextAreaState { focused, focus_visible: focused, ..Default::default() }, true);
        let textarea = textarea_style.root();
        let typography = self.look.typography_scale(ShadcnTextSize::Xs);

        compose_event_log_look(
            textarea.color("background"),
            textarea.color("foreground"),
            textarea.color("border"),
            textarea.color("placeholder"),
            typography,
            textarea.font_family_name().to_string(),
            rows,
            scale,
            textarea.number("border_width"),
        )
    }
}

pub trait EventLogViewLookExt {
    fn event_log_view(&self, id: impl Into<SharedString>) -> EventLogViewBuilder;
}

impl EventLogViewLookExt for Arc<ShadcnLook> {
    fn event_log_view(&self, id: impl Into<SharedString>) -> EventLogViewBuilder {
        EventLogViewBuilder::with_parts(id, shadcn_event_log_theme(Arc::clone(self)), self.scrollbar_template())
    }
}

pub fn shadcn_event_log_theme(look: Arc<ShadcnLook>) -> Arc<dyn EventLogTheme> {
    Arc::new(ShadcnEventLogTheme { look })
}

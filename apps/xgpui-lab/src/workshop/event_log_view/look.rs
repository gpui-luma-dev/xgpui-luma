use std::sync::Arc;

use gpui::SharedString;
use xgpui_luma::controls::textarea::TextAreaState;
use xgpui_luma::theme::{ControlSize, StandardBoxScale};
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};

use super::model::EventLogViewBuilder;
use super::theme::{EventLogColors, EventLogTheme, compose_event_log_look};

struct ShadcnEventLogTheme {
    look: Arc<ShadcnLook>,
}

impl EventLogTheme for ShadcnEventLogTheme {
    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.look.mode_tokens().metrics
    }

    fn resolve_look(&self, rows: usize, scale: &StandardBoxScale, focused: bool) -> super::theme::EventLogLook {
        let textarea = self.look.textarea_theme().resolve_look(
            TextAreaState { focused, focus_visible: focused, ..Default::default() },
            true,
            ControlSize::Md,
            scale,
        );
        let typography = self.look.typography_scale(ShadcnTextSize::Xs);

        compose_event_log_look(
            EventLogColors {
                background: textarea.background,
                foreground: textarea.foreground,
                border: textarea.border,
                placeholder: textarea.placeholder,
            },
            typography,
            textarea.font_family,
            rows,
            scale,
            textarea.border_width,
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

struct RadixEventLogTheme {
    look: Arc<xgpui_luma_look_radix::Look>,
}
impl EventLogTheme for RadixEventLogTheme {
    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.look.metrics()
    }
    fn resolve_look(&self, rows: usize, scale: &StandardBoxScale, focused: bool) -> super::theme::EventLogLook {
        use crate::theme::ColorVizLookExt;
        let textarea = xgpui_luma_look_radix::textarea_theme(&self.look).resolve_look(
            TextAreaState { focused, focus_visible: focused, ..Default::default() },
            true,
            ControlSize::Md,
            scale,
        );
        compose_event_log_look(
            EventLogColors {
                background: textarea.background,
                foreground: textarea.foreground,
                border: textarea.border,
                placeholder: textarea.placeholder,
            },
            self.look.typography_scale(crate::theme::TextSize::Xs),
            textarea.font_family,
            rows,
            scale,
            textarea.border_width,
        )
    }
}
pub fn radix_event_log_theme(look: Arc<xgpui_luma_look_radix::Look>) -> Arc<dyn EventLogTheme> {
    Arc::new(RadixEventLogTheme { look })
}
impl EventLogViewLookExt for Arc<xgpui_luma_look_radix::Look> {
    fn event_log_view(&self, id: impl Into<SharedString>) -> EventLogViewBuilder {
        use crate::theme::ColorVizLookExt;
        EventLogViewBuilder::with_parts(id, radix_event_log_theme(self.clone()), self.scrollbar_template())
    }
}

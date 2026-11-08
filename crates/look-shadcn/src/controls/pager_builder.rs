//! Look-owned pager builder. Spawn synthesizes the SDK [`xgpui_luma::controls::pager::Pager`].

use gpui::{App, Context, Entity, SharedString};
use xgpui_luma::controls::pager::{
    PagerBuilder, PagerControl, PagerIcons, PagerInfoSlot, PagerStyle, PagerTemplateParameters,
};

use crate::look::{ShadcnLook, resolve_look_from};

/// Builder in the guise of a pager: Shadcn theme plus SDK options, until `.spawn(cx)`.
pub struct Pager {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::pager::PagerEvent>,

    look: Option<ShadcnLook>,
    builder: PagerBuilder,
}

impl Pager {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::pager::PagerEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `PagerEvent::PageChanged` through the existing event stream.
    pub fn on_page_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::pager::PagerEvent::PageChanged { page } = event {
                callback(owner, *page, cx);
            }
        })
    }

    /// Handle `PagerEvent::PageSizeChanged` through the existing event stream.
    pub fn on_page_size_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::pager::PagerEvent::PageSizeChanged { page_size } = event {
                callback(owner, *page_size, cx);
            }
        })
    }

    /// Handle `PagerEvent::PageSizeOpenChanged` through the existing event stream.
    pub fn on_page_size_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::pager::PagerEvent::PageSizeOpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `PagerEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::pager::PagerEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self { callbacks: Default::default(), look: None, builder: xgpui_luma::controls::pager::new(id) }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn current_page(mut self, current_page: usize) -> Self {
        self.builder = self.builder.current_page(current_page);
        self
    }

    pub fn page_count(mut self, page_count: usize) -> Self {
        self.builder = self.builder.page_count(page_count);
        self
    }

    pub fn page_size(mut self, page_size: usize) -> Self {
        self.builder = self.builder.page_size(page_size);
        self
    }

    pub fn page_size_options(mut self, page_size_options: impl IntoIterator<Item = usize>) -> Self {
        self.builder = self.builder.page_size_options(page_size_options);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn icons(mut self, icons: PagerIcons) -> Self {
        self.builder = self.builder.icons(icons);
        self
    }

    pub fn style(mut self, style: PagerStyle) -> Self {
        self.builder = self.builder.style(style);
        self
    }

    pub fn info_text(mut self, info_text: impl Into<SharedString>) -> Self {
        self.builder = self.builder.info_text(info_text);
        self
    }

    pub fn info_slot(mut self, info_slot: PagerInfoSlot) -> Self {
        self.builder = self.builder.info_slot(info_slot);
        self
    }

    pub fn template_parameters(mut self, template_parameters: PagerTemplateParameters) -> Self {
        self.builder = self.builder.template_parameters(template_parameters);
        self
    }

    pub fn page_indicator_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(usize, usize) -> SharedString + Send + Sync + 'static,
    {
        self.builder = self.builder.page_indicator_formatter(formatter);
        self
    }

    pub fn page_size_label(mut self, label: impl Into<SharedString>) -> Self {
        self.builder = self.builder.page_size_label(label);
        self
    }

    pub fn page_size_trigger_width(mut self, width: f32) -> Self {
        self.builder = self.builder.page_size_trigger_width(width);
        self
    }

    pub fn first_label(mut self, label: impl Into<SharedString>) -> Self {
        self.builder = self.builder.first_label(label);
        self
    }

    pub fn previous_label(mut self, label: impl Into<SharedString>) -> Self {
        self.builder = self.builder.previous_label(label);
        self
    }

    pub fn next_label(mut self, label: impl Into<SharedString>) -> Self {
        self.builder = self.builder.next_label(label);
        self
    }

    pub fn last_label(mut self, label: impl Into<SharedString>) -> Self {
        self.builder = self.builder.last_label(label);
        self
    }

    pub fn show_first_last(mut self, show_first_last: bool) -> Self {
        self.builder = self.builder.show_first_last(show_first_last);
        self
    }

    pub fn numeric_slot_count(mut self, numeric_slot_count: usize) -> Self {
        self.builder = self.builder.numeric_slot_count(numeric_slot_count);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<PagerControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { self.into_sdk_builder(cx).spawn(cx) };
        callbacks.bind(&control, cx);
        control
    }

    /// Convert to the SDK builder using `.look(...)`, ambient Global, or [`ShadcnLook::built_in()`].
    ///
    /// Use this when a look-owned pager is composed into another SDK builder
    /// (for example `paging_table!`) that expects [`PagerBuilder`]. Pass `cx`
    /// so ambient look resolution works when `.look(...)` was omitted.
    pub fn into_sdk_builder(self, cx: &App) -> PagerBuilder {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.into_sdk_builder_with(look)
    }

    fn into_sdk_builder_with(self, look: ShadcnLook) -> PagerBuilder {
        self.builder.stylesheet(&look.stylesheet().common).theme(look.pager_theme())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Pager::new("ok")
            .look(&look)
            .style(PagerStyle::MinimalEdge)
            .page_count(4)
            .into_sdk_builder_with(look);
    }
}

use gpui::{ClickEvent, Context, EventEmitter, IntoElement, MouseDownEvent, Render, SharedString, Window, div, prelude::*};

use super::model::{PagerBuilder, PagerInfoSlot, PagerModel, PagerRenderModel, PagerTemplateParameters};
use super::template::PagerTemplateHandlers;
use crate::theme::observe_theme_revision;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PagerEvent {
    PageChanged { page: usize },
    PageSizeChanged { page_size: usize },
    PageSizeOpenChanged { open: bool },
    EnabledChanged { enabled: bool },
}

pub struct PagerControl {
    model: PagerModel,
    page_size_open: bool,
}

impl EventEmitter<PagerEvent> for PagerControl {}

impl PagerControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> PagerBuilder {
        PagerBuilder::new(id)
    }

    pub(crate) fn from_builder(mut builder: PagerBuilder, cx: &mut Context<Self>) -> Self {
        normalize_model(&mut builder.model);
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self { model: builder.model, page_size_open: false }
    }

    pub fn current_page(&self) -> usize {
        self.model.current_page
    }

    pub fn page_count(&self) -> usize {
        self.model.page_count
    }

    pub fn page_size(&self) -> usize {
        self.model.page_size
    }

    pub fn page_size_options(&self) -> &[usize] {
        &self.model.page_size_options
    }

    pub fn style(&self) -> super::PagerStyle {
        self.model.style
    }

    pub fn is_enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn set_page(&mut self, page: usize, cx: &mut Context<Self>) {
        self.set_page_internal(page, false, cx);
    }

    pub fn first_page(&mut self, cx: &mut Context<Self>) {
        self.set_page(0, cx);
    }

    pub fn prev_page(&mut self, cx: &mut Context<Self>) {
        self.set_page(self.model.current_page.saturating_sub(1), cx);
    }

    pub fn next_page(&mut self, cx: &mut Context<Self>) {
        self.set_page(self.model.current_page.saturating_add(1), cx);
    }

    pub fn last_page(&mut self, cx: &mut Context<Self>) {
        self.set_page(self.model.page_count.saturating_sub(1), cx);
    }

    pub fn set_page_count(&mut self, page_count: usize, cx: &mut Context<Self>) {
        let next_page_count = page_count;
        let next_page = clamp_page(self.model.current_page, next_page_count);
        if self.model.page_count == next_page_count && self.model.current_page == next_page {
            return;
        }

        self.model.page_count = next_page_count;
        self.model.current_page = next_page;
        cx.notify();
    }

    pub fn set_page_size(&mut self, page_size: usize, cx: &mut Context<Self>) {
        self.set_page_size_internal(page_size, false, cx);
    }

    pub fn set_page_size_options(
        &mut self,
        page_size_options: impl IntoIterator<Item = usize>,
        cx: &mut Context<Self>,
    ) {
        let next: Vec<usize> = page_size_options.into_iter().map(|size| size.max(1)).collect();
        if self.model.page_size_options == next {
            return;
        }

        self.model.page_size_options = next;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        if !enabled {
            self.set_page_size_open(false, true, cx);
        }
        cx.emit(PagerEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_style(&mut self, style: super::PagerStyle, cx: &mut Context<Self>) {
        if self.model.style == style {
            return;
        }

        self.model.style = style;
        self.page_size_open = false;
        cx.notify();
    }

    pub fn set_info_text(&mut self, info_text: Option<impl Into<SharedString>>, cx: &mut Context<Self>) {
        let next = info_text.map(Into::into);
        if self.model.info_text == next {
            return;
        }

        self.model.info_text = next;
        cx.notify();
    }

    pub fn set_info_slot(&mut self, info_slot: Option<PagerInfoSlot>, cx: &mut Context<Self>) {
        self.model.info_slot = info_slot;
        cx.notify();
    }

    pub fn set_template_parameters(&mut self, template_parameters: PagerTemplateParameters, cx: &mut Context<Self>) {
        self.model.template_parameters = template_parameters;
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::PagerTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn super::PagerTheme>, cx: &mut Context<Self>) {
        self.model.theme = Some(theme.clone());
        self.set_template(std::sync::Arc::new(super::ThemedPagerTemplate::new(theme)), cx);
    }

    fn render_model(&self) -> PagerRenderModel<'_> {
        PagerRenderModel {
            id: &self.model.id,
            current_page: self.model.current_page,
            page_count: self.model.page_count,
            page_size: self.model.page_size,
            page_size_options: &self.model.page_size_options,
            page_size_open: self.page_size_open,
            enabled: self.model.enabled,
            style: self.model.style,
            info_text: self.model.info_text.as_ref(),
            info_slot: self.model.info_slot.as_ref(),
            template_parameters: &self.model.template_parameters,
            icons: &self.model.icons,
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> PagerTemplateHandlers {
        let entity = cx.entity();
        let page_entity = entity.clone();
        let page_size_entity = entity.clone();

        PagerTemplateHandlers {
            outside_mouse_down: std::sync::Arc::new(cx.listener(Self::handle_outside_mouse_down)),
            toggle_page_size: std::sync::Arc::new(cx.listener(Self::handle_toggle_page_size)),
            set_page: std::sync::Arc::new(move |page, _event, _window, cx| {
                page_entity.update(cx, |this, cx| {
                    if this.model.enabled {
                        this.set_page_internal(page, true, cx);
                    }
                });
            }),
            set_page_size: std::sync::Arc::new(move |page_size, _event, _window, cx| {
                page_size_entity.update(cx, |this, cx| {
                    if this.model.enabled {
                        this.set_page_size_internal(page_size, true, cx);
                    }
                });
            }),
        }
    }

    fn handle_outside_mouse_down(&mut self, _: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.page_size_open && self.model.dismiss.click_outside {
            self.set_page_size_open(false, true, cx);
        }
    }

    fn handle_toggle_page_size(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || self.model.page_size_options.is_empty() {
            return;
        }

        self.set_page_size_open(!self.page_size_open, true, cx);
    }

    fn set_page_internal(&mut self, page: usize, emit: bool, cx: &mut Context<Self>) {
        let next = clamp_page(page, self.model.page_count);
        if self.model.current_page == next {
            return;
        }

        self.model.current_page = next;
        if emit {
            cx.emit(PagerEvent::PageChanged { page: next });
        }
        cx.notify();
    }

    fn set_page_size_internal(&mut self, page_size: usize, emit: bool, cx: &mut Context<Self>) {
        let page_size = page_size.max(1);
        if self.model.page_size == page_size {
            // Choosing the current size still closes the menu.
            self.set_page_size_open(false, emit, cx);
            return;
        }

        self.model.page_size = page_size;
        self.set_page_size_open(false, emit, cx);
        if emit {
            cx.emit(PagerEvent::PageSizeChanged { page_size });
        }
        cx.notify();
    }

    fn set_page_size_open(&mut self, open: bool, emit: bool, cx: &mut Context<Self>) {
        if self.page_size_open == open {
            return;
        }

        self.page_size_open = open;
        if emit {
            cx.emit(PagerEvent::PageSizeOpenChanged { open });
        }
        cx.notify();
    }
}

impl Render for PagerControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model();
        let handlers = self.template_handlers(cx);
        div().child(self.model.template.render(&model, handlers, window, cx)).into_any_element()
    }
}

fn normalize_model(model: &mut PagerModel) {
    model.page_size = model.page_size.max(1);
    model.current_page = clamp_page(model.current_page, model.page_count);
    if model.page_size_options.contains(&0) {
        model.page_size_options = model.page_size_options.iter().copied().map(|size| size.max(1)).collect();
    }
}

fn clamp_page(page: usize, page_count: usize) -> usize {
    if page_count == 0 { 0 } else { page.min(page_count - 1) }
}

#[cfg(test)]
mod tests {
    use super::clamp_page;

    #[test]
    fn clamp_page_respects_upper_bound() {
        assert_eq!(clamp_page(8, 3), 2);
    }

    #[test]
    fn clamp_page_handles_empty_counts() {
        assert_eq!(clamp_page(4, 0), 0);
    }
}

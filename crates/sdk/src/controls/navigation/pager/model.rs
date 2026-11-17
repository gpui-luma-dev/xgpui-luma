use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, AppContext, SharedString, Window};

use super::control::PagerControl;
use super::template::{PagerTemplate, ThemedPagerTemplate, default_pager_template, template_with_modifier};
use super::theme::PagerTheme;
use crate::infra::icon::IconSource;

pub type PagerInfoSlot =
    Arc<dyn for<'a> Fn(&PagerRenderModel<'a>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static>;
pub type PagerPageIndicatorFormatter = Arc<dyn Fn(usize, usize) -> SharedString + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PagerStyle {
    Minimal,
    #[default]
    MinimalEdge,
    Numeric,
}

impl PagerStyle {
    /// Stylesheet key for per-presentation geometry overrides.
    pub fn stylesheet_key(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::MinimalEdge => "minimal-edge",
            Self::Numeric => "numeric",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PagerPageItem {
    Page(usize),
    Gap { target: usize },
}

#[derive(Clone)]
pub struct PagerTemplateParameters {
    pub page_indicator_formatter: PagerPageIndicatorFormatter,
    pub page_size_label: Option<SharedString>,
    pub page_size_trigger_width: f32,
    pub first_label: Option<SharedString>,
    pub previous_label: Option<SharedString>,
    pub next_label: Option<SharedString>,
    pub last_label: Option<SharedString>,
    pub show_first_last: bool,
    pub numeric_slot_count: usize,
}

// Share the compatibility formatter so locale-aware rendering can distinguish it
// from app callbacks, including callbacks assigned directly to the public field.
fn default_page_indicator_formatter() -> PagerPageIndicatorFormatter {
    static FORMATTER: OnceLock<PagerPageIndicatorFormatter> = OnceLock::new();
    FORMATTER.get_or_init(|| Arc::new(|page, pages| format!("Page {page} of {pages}").into())).clone()
}

impl Default for PagerTemplateParameters {
    fn default() -> Self {
        Self {
            page_indicator_formatter: default_page_indicator_formatter(),
            page_size_label: None,
            page_size_trigger_width: crate::theme::stylesheet::CommonStylesheet::sdk_default()
                .shared_geometry()
                .page_size_trigger_width,
            first_label: None,
            previous_label: None,
            next_label: None,
            last_label: None,
            show_first_last: true,
            numeric_slot_count: crate::theme::stylesheet::CommonStylesheet::sdk_default().row_limits.resolve("pager").1,
        }
    }
}

impl PagerTemplateParameters {
    pub fn page_indicator_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(usize, usize) -> SharedString + Send + Sync + 'static,
    {
        self.page_indicator_formatter = Arc::new(formatter);
        self
    }

    pub fn page_size_label(mut self, label: impl Into<SharedString>) -> Self {
        self.page_size_label = Some(label.into());
        self
    }

    pub fn page_size_label_opt(mut self, label: Option<impl Into<SharedString>>) -> Self {
        self.page_size_label = label.map(Into::into);
        self
    }

    pub fn page_size_trigger_width(mut self, width: f32) -> Self {
        self.page_size_trigger_width = width;
        self
    }

    pub fn first_label(mut self, label: impl Into<SharedString>) -> Self {
        self.first_label = Some(label.into());
        self
    }

    pub fn previous_label(mut self, label: impl Into<SharedString>) -> Self {
        self.previous_label = Some(label.into());
        self
    }

    pub fn next_label(mut self, label: impl Into<SharedString>) -> Self {
        self.next_label = Some(label.into());
        self
    }

    pub fn last_label(mut self, label: impl Into<SharedString>) -> Self {
        self.last_label = Some(label.into());
        self
    }

    pub fn show_first_last(mut self, show_first_last: bool) -> Self {
        self.show_first_last = show_first_last;
        self
    }

    pub fn numeric_slot_count(mut self, numeric_slot_count: usize) -> Self {
        self.numeric_slot_count = numeric_slot_count;
        self
    }
}

#[derive(Clone)]
pub struct PagerModel {
    pub(crate) id: SharedString,
    pub(crate) current_page: usize,
    pub(crate) page_count: usize,
    pub(crate) page_size: usize,
    pub(crate) page_size_options: Vec<usize>,
    pub(crate) enabled: bool,
    pub(crate) style: PagerStyle,
    pub(crate) info_text: Option<SharedString>,
    pub(crate) info_slot: Option<PagerInfoSlot>,
    pub(crate) template_parameters: PagerTemplateParameters,
    pub(crate) numeric_slot_count_explicit: bool,
    pub(crate) page_size_trigger_width_explicit: bool,
    pub(crate) icons_explicit: bool,
    pub(crate) icons: PagerIcons,
    pub(crate) theme: Option<Arc<dyn PagerTheme>>,
    pub(crate) template: Arc<dyn PagerTemplate>,
}

#[derive(Clone, Debug)]
pub struct PagerIcons {
    /// Ellipsis or other gap-jump glyph.
    pub gap: IconSource,
    pub page_size_open: IconSource,
    pub page_size_closed: IconSource,
    pub first: IconSource,
    pub previous: IconSource,
    pub next: IconSource,
    pub last: IconSource,
}

impl Default for PagerIcons {
    fn default() -> Self {
        Self::from_stylesheet(crate::theme::stylesheet::CommonStylesheet::sdk_default())
    }
}
impl PagerIcons {
    /// Resolve default icon roles before applying explicit consumer icon overrides.
    pub fn from_stylesheet(sheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        use crate::theme::stylesheet::IconRole;
        Self {
            gap: sheet.icons.resolve(IconRole::PagerGap),
            page_size_open: sheet.icons.resolve(IconRole::MenuExpanded),
            page_size_closed: sheet.icons.resolve(IconRole::MenuCollapsed),
            first: sheet.icons.resolve(IconRole::PagerFirst),
            previous: sheet.icons.resolve(IconRole::PagerPrevious),
            next: sheet.icons.resolve(IconRole::PagerNext),
            last: sheet.icons.resolve(IconRole::PagerLast),
        }
    }
}

pub struct PagerRenderModel<'a> {
    pub id: &'a SharedString,
    pub current_page: usize,
    pub page_count: usize,
    pub page_size: usize,
    pub page_size_options: &'a [usize],
    pub page_size_open: bool,
    pub enabled: bool,
    pub style: PagerStyle,
    pub info_text: Option<&'a SharedString>,
    pub info_slot: Option<&'a PagerInfoSlot>,
    pub template_parameters: &'a PagerTemplateParameters,
    pub icons: &'a PagerIcons,
}

impl PagerRenderModel<'_> {
    pub fn display_current_page(&self) -> usize {
        if self.page_count == 0 {
            1
        } else {
            self.current_page.min(self.page_count.saturating_sub(1)) + 1
        }
    }

    pub fn display_page_count(&self) -> usize {
        self.page_count.max(1)
    }

    /// Format without an app context, using the compatibility English default.
    pub fn page_indicator(&self) -> SharedString {
        (self.template_parameters.page_indicator_formatter)(self.display_current_page(), self.display_page_count())
    }

    /// Format the built-in indicator with the active SDK locale. App callbacks
    /// take precedence, including direct assignments to template parameters.
    /// Custom templates should use this method to follow locale changes.
    pub fn page_indicator_with_locale(&self, cx: &App) -> SharedString {
        if Arc::ptr_eq(&self.template_parameters.page_indicator_formatter, &default_page_indicator_formatter()) {
            (crate::infra::locale::LocaleContext::current(cx).strings.page_of)(
                self.display_current_page(),
                self.display_page_count(),
                cx,
            )
        } else {
            self.page_indicator()
        }
    }

    pub fn page_size_label(&self) -> Option<&SharedString> {
        self.template_parameters.page_size_label.as_ref()
    }

    pub fn page_size_trigger_width(&self) -> f32 {
        let width = self.template_parameters.page_size_trigger_width;
        if width.is_finite() && width > 0.0 {
            width
        } else {
            crate::theme::stylesheet::CommonStylesheet::sdk_default().shared_geometry().page_size_trigger_width
        }
    }

    pub fn first_label(&self) -> Option<&SharedString> {
        self.template_parameters.first_label.as_ref()
    }

    pub fn previous_label(&self) -> Option<&SharedString> {
        self.template_parameters.previous_label.as_ref()
    }

    pub fn next_label(&self) -> Option<&SharedString> {
        self.template_parameters.next_label.as_ref()
    }

    pub fn last_label(&self) -> Option<&SharedString> {
        self.template_parameters.last_label.as_ref()
    }

    pub fn show_first_last(&self) -> bool {
        self.template_parameters.show_first_last
    }

    pub fn numeric_slot_count(&self) -> usize {
        self.template_parameters.numeric_slot_count.max(5)
    }

    pub fn at_first(&self) -> bool {
        self.current_page == 0
    }

    pub fn at_last(&self) -> bool {
        self.current_page + 1 >= self.page_count.max(1)
    }
}

pub struct PagerBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::pager::PagerEvent>,

    pub(crate) model: PagerModel,
}

impl PagerBuilder {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::pager::PagerEvent, &mut gpui::Context<M>) + 'static,
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
            if let crate::controls::pager::PagerEvent::PageChanged { page } = event {
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
            if let crate::controls::pager::PagerEvent::PageSizeChanged { page_size } = event {
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
            if let crate::controls::pager::PagerEvent::PageSizeOpenChanged { open } = event {
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
            if let crate::controls::pager::PagerEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: PagerModel {
                id: id.into(),
                current_page: 0,
                page_count: 0,
                page_size: 10,
                page_size_options: vec![10, 25],
                enabled: true,
                style: PagerStyle::MinimalEdge,
                info_text: None,
                info_slot: None,
                template_parameters: PagerTemplateParameters::default(),
                numeric_slot_count_explicit: false,
                page_size_trigger_width_explicit: false,
                icons_explicit: false,
                icons: PagerIcons::default(),
                theme: None,
                template: default_pager_template(),
            },
        }
    }

    pub fn current_page(mut self, current_page: usize) -> Self {
        self.model.current_page = current_page;
        self
    }

    pub fn page_count(mut self, page_count: usize) -> Self {
        self.model.page_count = page_count;
        self
    }

    pub fn page_size(mut self, page_size: usize) -> Self {
        self.model.page_size = page_size.max(1);
        self
    }

    pub fn page_size_options(mut self, page_size_options: impl IntoIterator<Item = usize>) -> Self {
        self.model.page_size_options = page_size_options.into_iter().map(|size| size.max(1)).collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    /// Capture default glyphs; explicit `.icons(...)` wins in either method order.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        if !self.model.numeric_slot_count_explicit {
            self.model.template_parameters.numeric_slot_count = stylesheet.row_limits.resolve("pager").1;
        }
        if !self.model.page_size_trigger_width_explicit {
            self.model.template_parameters.page_size_trigger_width =
                stylesheet.shared_geometry().page_size_trigger_width;
        }
        if !self.model.icons_explicit {
            self.model.icons = PagerIcons::from_stylesheet(stylesheet);
        }
        self
    }

    pub fn icons(mut self, icons: PagerIcons) -> Self {
        self.model.icons_explicit = true;
        self.model.icons = icons;
        self
    }

    pub fn style(mut self, style: PagerStyle) -> Self {
        self.model.style = style;
        self
    }

    pub fn info_text(mut self, info_text: impl Into<SharedString>) -> Self {
        self.model.info_text = Some(info_text.into());
        self
    }

    pub fn info_text_opt(mut self, info_text: Option<impl Into<SharedString>>) -> Self {
        self.model.info_text = info_text.map(Into::into);
        self
    }

    pub fn info_slot(mut self, info_slot: PagerInfoSlot) -> Self {
        self.model.info_slot = Some(info_slot);
        self
    }

    pub fn template_parameters(mut self, template_parameters: PagerTemplateParameters) -> Self {
        self.model.page_size_trigger_width_explicit = true;
        self.model.numeric_slot_count_explicit = true;
        self.model.template_parameters = template_parameters;
        self
    }

    pub fn page_indicator_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(usize, usize) -> SharedString + Send + Sync + 'static,
    {
        self.model.template_parameters.page_indicator_formatter = Arc::new(formatter);
        self
    }

    pub fn page_size_label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.template_parameters.page_size_label = Some(label.into());
        self
    }

    pub fn page_size_label_opt(mut self, label: Option<impl Into<SharedString>>) -> Self {
        self.model.template_parameters.page_size_label = label.map(Into::into);
        self
    }

    pub fn page_size_trigger_width(mut self, width: f32) -> Self {
        self.model.page_size_trigger_width_explicit = true;
        self.model.template_parameters.page_size_trigger_width = width;
        self
    }

    pub fn first_label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.template_parameters.first_label = Some(label.into());
        self
    }

    pub fn previous_label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.template_parameters.previous_label = Some(label.into());
        self
    }

    pub fn next_label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.template_parameters.next_label = Some(label.into());
        self
    }

    pub fn last_label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.template_parameters.last_label = Some(label.into());
        self
    }

    pub fn show_first_last(mut self, show_first_last: bool) -> Self {
        self.model.template_parameters.show_first_last = show_first_last;
        self
    }

    pub fn numeric_slot_count(mut self, numeric_slot_count: usize) -> Self {
        self.model.numeric_slot_count_explicit = true;
        self.model.template_parameters.numeric_slot_count = numeric_slot_count;
        self
    }

    pub fn template(mut self, template: Arc<dyn PagerTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &PagerRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn theme(mut self, theme: Arc<dyn PagerTheme>) -> Self {
        self.model.theme = Some(theme.clone());
        self.model.template = Arc::new(ThemedPagerTemplate::new(theme));
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> gpui::Entity<PagerControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| PagerControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

pub fn new(id: impl Into<SharedString>) -> PagerBuilder {
    PagerBuilder::new(id)
}

#[cfg(test)]
mod tests {
    use super::{PagerBuilder, PagerIcons, PagerRenderModel, PagerTemplateParameters, default_pager_template};
    use gpui::SharedString;

    #[test]
    fn page_indicator_uses_template_formatter() {
        let params = PagerTemplateParameters::default()
            .page_indicator_formatter(|current, total| SharedString::from(format!("{current}/{total}")));
        let id = SharedString::from("pager");
        let model = PagerRenderModel {
            id: &id,
            current_page: 2,
            page_count: 12,
            page_size: 25,
            page_size_options: &[],
            page_size_open: false,
            enabled: true,
            style: super::PagerStyle::Minimal,
            info_text: None,
            info_slot: None,
            template_parameters: &params,
            icons: &PagerIcons::default(),
        };

        assert_eq!(model.page_indicator(), SharedString::from("3/12"));
    }

    #[test]
    fn nav_labels_round_trip_from_template_parameters() {
        let params = PagerTemplateParameters::default()
            .first_label("First")
            .previous_label("Back")
            .next_label("Next")
            .last_label("Last");
        let id = SharedString::from("pager");
        let model = PagerRenderModel {
            id: &id,
            current_page: 0,
            page_count: 3,
            page_size: 10,
            page_size_options: &[],
            page_size_open: false,
            enabled: true,
            style: super::PagerStyle::MinimalEdge,
            info_text: None,
            info_slot: None,
            template_parameters: &params,
            icons: &PagerIcons::default(),
        };

        assert_eq!(model.first_label(), Some(&SharedString::from("First")));
        assert_eq!(model.previous_label(), Some(&SharedString::from("Back")));
        assert_eq!(model.next_label(), Some(&SharedString::from("Next")));
        assert_eq!(model.last_label(), Some(&SharedString::from("Last")));
    }

    #[test]
    fn numeric_slot_count_is_clamped_to_minimum() {
        let params = PagerTemplateParameters::default().numeric_slot_count(3).show_first_last(false);
        let id = SharedString::from("pager");
        let model = PagerRenderModel {
            id: &id,
            current_page: 0,
            page_count: 10,
            page_size: 10,
            page_size_options: &[],
            page_size_open: false,
            enabled: true,
            style: super::PagerStyle::Numeric,
            info_text: None,
            info_slot: None,
            template_parameters: &params,
            icons: &PagerIcons::default(),
        };

        assert_eq!(model.numeric_slot_count(), 5);
        assert!(!model.show_first_last());
    }

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_pager_template();
        let builder = PagerBuilder::new("pager-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!std::sync::Arc::ptr_eq(&builder.model.template, &template));
    }
}

#[cfg(test)]
mod icon_precedence_tests {
    use super::*;
    #[test]
    fn explicit_icons_win_before_and_after_stylesheet_capture() {
        let sheet =
            crate::theme::stylesheet::CommonStylesheet::parse("[common.icons]\npager_next='arrow-right'").unwrap();
        let custom = PagerIcons { next: crate::infra::icon::IconSource::from("custom-next.svg"), ..Default::default() };
        let before = PagerBuilder::new("before").icons(custom.clone()).stylesheet(&sheet);
        let after = PagerBuilder::new("after").stylesheet(&sheet).icons(custom);
        assert_eq!(before.model.icons.next, after.model.icons.next);
        assert_eq!(before.model.icons.next.svg_path().unwrap().as_ref(), "custom-next.svg");
        let inherited = PagerBuilder::new("default").stylesheet(&sheet);
        assert_eq!(inherited.model.icons.next.lucide(), Some(lucide_svg_static::Icon::ArrowRight));
    }
}

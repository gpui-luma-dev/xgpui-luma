//! Generic, read-only theme inspector (issue #282 spike).
//!
//! Works for any `#[derive(Style)]` struct: rows come from the style's
//! descriptors, values from `Style::get`, and sources from `Style::describe`.
//! State toggles are generated from the states the style's rules use.
//! [`InspectorSplit`] places a sample and an inspector side by side.
use std::sync::Arc;

use gpui::{AnyElement, AnyView, Context, Entity, Hsla, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::resizable_panels::{
    ResizablePanelSpec, ResizablePanels, ResizablePanelsBuilder, ResizablePanelsOrientation,
};
use xgpui_luma::controls::button::HasPresenter;
use xgpui_luma::controls::toggle::{self, ToggleEvent};
use xgpui_luma::theme::ThemeTokens;
use xgpui_luma::theme::style_tokens::ThemeTokenSource;
use xgpui_luma_style::{Dyn, FieldContent, FieldDesc, ResolveCx, State, StateSet, Style, ValueKind};

use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};

/// Row groups, in display order.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Category {
    Colors,
    Layout,
    Typography,
    Elevation,
}

impl Category {
    const ALL: [Category; 4] = [Category::Colors, Category::Layout, Category::Typography, Category::Elevation];

    fn title(self) -> &'static str {
        match self {
            Self::Colors => "Colors",
            Self::Layout => "Layout",
            Self::Typography => "Typography",
            Self::Elevation => "Elevation",
        }
    }

    fn of(desc: &FieldDesc) -> Option<Self> {
        let FieldContent::Value { kind, .. } = &desc.content else {
            return None;
        };
        Some(match kind.value {
            ValueKind::Color => Self::Colors,
            ValueKind::Shadow => Self::Elevation,
            ValueKind::FontFamily | ValueKind::FontWeight => Self::Typography,
            ValueKind::Px | ValueKind::Scalar | ValueKind::Opacity => Self::Layout,
        })
    }
}

/// A read-only inspector for one style resolved against the SDK theme tokens.
pub(super) struct ThemeInspector<S: Style> {
    look: Arc<Look>,
    title: &'static str,
    family: &'static str,
    tokens: ThemeTokens,
    style: S,
    at: StateSet,
    toggles: Vec<toggle::Toggle>,
}

impl<S: Style> ThemeInspector<S> {
    /// `family` selects the SDK palette recipes (e.g. `"checkbox"`).
    pub(super) fn new(
        look: Arc<Look>,
        title: &'static str,
        family: &'static str,
        tokens: ThemeTokens,
        style: S,
        cx: &mut Context<Self>,
    ) -> Self {
        let toggles = style
            .states_used()
            .iter()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|state| {
                let toggle = toggle::new(format!("theme-inspector-{family}-{}", state.name()))
                    .content(move |_, _| div().child(state_label(state)))
                    .spawn(cx);
                cx.subscribe(&toggle, move |this: &mut Self, _, event: &ToggleEvent, cx| {
                    if let ToggleEvent::Change { selected } = event {
                        this.at = if *selected {
                            this.at.with(state)
                        } else {
                            this.at.without(state)
                        };
                        cx.notify();
                    }
                })
                .detach();
                toggle
            })
            .collect();
        Self { look, title, family, tokens, style, at: StateSet::NONE, toggles }
    }

    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }

    /// Current states, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn states(&self) -> StateSet {
        self.at
    }

    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn toggle_count(&self) -> usize {
        self.toggles.len()
    }

    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn toggle(&self, index: usize) -> toggle::Toggle {
        self.toggles[index].clone()
    }

    fn row(&self, path: &str, desc: &FieldDesc, cx: &ResolveCx<'_>) -> AnyElement {
        let chrome = self.look.chrome();
        let kind = match &desc.content {
            FieldContent::Value { kind, .. } => kind.value,
            FieldContent::Part(_) => return div().into_any_element(),
        };
        let value = self.style.get(path, cx, self.at);
        let source = self.style.describe(path, self.at).unwrap_or_default();
        let swatch = match &value {
            Some(Dyn::Color(color)) => Some(*color),
            _ => None,
        };
        div()
            .w_full()
            .flex()
            .items_start()
            .gap(px(12.))
            .py(px(4.))
            .border_b_1()
            .border_color(chrome.border)
            .child(div().w(px(150.)).flex_shrink_0().child(path.to_owned()))
            .child(
                div()
                    .w(px(130.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .children(swatch.map(swatch_box))
                    .child(format_value(value.as_ref(), kind)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .font_family(crate::theme::MONO_FAMILY)
                    .text_color(chrome.muted_text)
                    .child(source),
            )
            .into_any_element()
    }
}

impl<S: Style> Render for ThemeInspector<S> {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let small = self.look.typography_scale(TextSize::Xs);
        let source = ThemeTokenSource::for_family(&self.tokens, self.family);
        let resolve = ResolveCx::new(&source, 1.0);
        let leaves = S::SHAPE.leaves();

        let sections = Category::ALL.into_iter().filter_map(|category| {
            let rows: Vec<AnyElement> = leaves
                .iter()
                .filter(|(_, desc)| Category::of(desc) == Some(category))
                .map(|(path, desc)| self.row(path, desc, &resolve))
                .collect();
            (!rows.is_empty()).then(|| {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(div().pt(px(8.)).text_color(chrome.title_text).child(category.title()))
                    .children(rows)
            })
        });

        div()
            .id("theme-inspector")
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(div().text_color(chrome.title_text).child(format!("Theme Inspector · {}", self.title)))
            .child(div().flex().flex_wrap().gap(px(8.)).children(self.toggles.iter().cloned()))
            .child(div().typography_style(small).text_color(chrome.muted_text).child(format!("States: {}", self.at)))
            .child(div().typography_style(small).flex().flex_col().children(sections))
    }
}

fn state_label(state: State) -> &'static str {
    match state {
        State::Invalid => "Invalid",
        State::Focused => "Focused",
        State::Selected => "Selected",
        State::Checked => "Checked",
        State::Hovered => "Hovered",
        State::Pressed => "Pressed",
        State::Disabled => "Disabled",
    }
}

fn swatch_box(color: Hsla) -> AnyElement {
    div().size(px(12.)).flex_shrink_0().rounded(px(2.)).border_1().bg(color).into_any_element()
}

fn format_value(value: Option<&Dyn>, kind: ValueKind) -> String {
    match value {
        None => "—".into(),
        Some(Dyn::Number(number)) => match kind {
            ValueKind::Px => format!("{number:.1}px"),
            ValueKind::FontWeight => format!("{number:.0}"),
            _ => format!("{number:.2}"),
        },
        Some(Dyn::Color(color)) => {
            let rgba = gpui::Rgba::from(*color);
            let byte = |channel: f32| (channel.clamp(0.0, 1.0) * 255.0).round() as u8;
            format!("#{:02x}{:02x}{:02x}{:02x}", byte(rgba.r), byte(rgba.g), byte(rgba.b), byte(rgba.a))
        }
        Some(Dyn::Shadow(layers)) => format!("{} layer(s)", layers.len()),
        Some(Dyn::Text(text)) => text.to_string(),
        Some(Dyn::None) => "none".into(),
    }
}

/// A sample and its inspector side by side in resizable panels.
pub(super) struct InspectorSplit {
    panels: Entity<ResizablePanels>,
}

impl InspectorSplit {
    pub(super) fn new(id: &'static str, sample: AnyView, inspector: AnyView, cx: &mut Context<Self>) -> Self {
        let panels = ResizablePanelsBuilder::new(id)
            .orientation(ResizablePanelsOrientation::Horizontal)
            .panel(
                ResizablePanelSpec::new_render(move || div().size_full().min_w_0().child(sample.clone()))
                    .weight(3.)
                    .min(px(360.)),
            )
            .panel(
                ResizablePanelSpec::new_render(move || div().size_full().min_w_0().child(inspector.clone()))
                    .weight(2.)
                    .min(px(420.)),
            )
            .spawn(cx);
        Self { panels }
    }
}

impl Render for InspectorSplit {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.panels.clone())
    }
}

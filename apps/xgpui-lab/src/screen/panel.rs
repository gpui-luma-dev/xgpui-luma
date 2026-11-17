//! Supporting panels for the two-panel row.
//!
//! A free-body `PanelShell` is added when a screen needs one; see `docs/lab-screen-api.md`.
use crate::l10n::UiCopy;
use gpui::{AnyElement, Div, Hsla, IntoElement, ParentElement, Styled, div, relative};
use xgpui_luma::{hstack, vstack};
use xgpui_luma_look_radix::Look;

use super::{LabMetrics, TextRole, panel_surface, readout, text};
use crate::theme::ColorVizLookExt;
use gpui::{prelude::FluentBuilder, px};

pub(crate) enum Panel {
    Parameters(ParametersPanel),
    Preview(PreviewPanel),
    Chain(ChainPanel),
    Custom(PanelShell),
}

impl Panel {
    pub(super) fn render(self, look: &Look, metrics: &LabMetrics) -> Div {
        match self {
            Self::Parameters(panel) => panel.render(look, metrics),
            Self::Preview(panel) => panel.render(look, metrics),
            Self::Chain(panel) => panel.render(look),
            Self::Custom(panel) => panel.render(look),
        }
    }
}

/// Panel surface with its heading: title and subtitle stack with no gap between them,
/// with an optional header action (e.g. *Reset filters*) at the right.
fn shell(look: &Look, title: UiCopy, subtitle: Option<UiCopy>, action: Option<AnyElement>) -> Div {
    let heading = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .child(text(look, TextRole::Section, title))
        .children(subtitle.map(|subtitle| text(look, TextRole::Caption, subtitle)));
    let header = div().flex().items_start().gap_3().child(heading).children(action);
    panel_surface(look).child(header)
}

/// One labeled parameter: label and optional range, then a text input above its slider.
pub(crate) struct Param {
    label: UiCopy,
    range: Option<UiCopy>,
    input: AnyElement,
    slider: AnyElement,
}

impl Param {
    pub fn new(label: impl Into<UiCopy>, input: impl IntoElement, slider: impl IntoElement) -> Self {
        Self { label: label.into(), range: None, input: input.into_any_element(), slider: slider.into_any_element() }
    }

    /// The accepted range, shown muted at the right of the label, e.g. `0–2` or `0–360°`.
    pub fn range(mut self, range: impl Into<UiCopy>) -> Self {
        self.range = Some(range.into());
        self
    }
}

pub(crate) struct ParametersPanel {
    title: UiCopy,
    subtitle: Option<UiCopy>,
    params: Vec<Param>,
    footer: Option<AnyElement>,
    action: Option<AnyElement>,
}

impl FluentBuilder for ParametersPanel {}

impl ParametersPanel {
    pub fn new(title: impl Into<UiCopy>) -> Self {
        Self { title: title.into(), subtitle: None, params: Vec::new(), footer: None, action: None }
    }

    pub fn subtitle(mut self, text: impl Into<UiCopy>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    /// Parameters in a wrapping row of equal columns.
    pub fn params(mut self, params: impl IntoIterator<Item = Param>) -> Self {
        self.params = params.into_iter().collect();
        self
    }

    /// A control at the right of the panel heading, e.g. a *Reset filters* button.
    pub fn action(mut self, element: impl IntoElement) -> Self {
        self.action = Some(element.into_any_element());
        self
    }

    /// A row under the parameters, e.g. preset buttons.
    pub fn footer(mut self, element: impl IntoElement) -> Self {
        self.footer = Some(element.into_any_element());
        self
    }

    fn render(self, look: &Look, metrics: &LabMetrics) -> Div {
        let min_width = metrics.param_min_width;
        // At most three columns: each parameter grows from a basis just under a third.
        let column = |param: Param| {
            let label = crate::l10n::row(
                div()
                    .flex()
                    .items_baseline()
                    .justify_between()
                    .gap_2()
                    .child(text(look, TextRole::Field, param.label))
                    .children(param.range.map(|range| text(look, TextRole::Label, range))),
            );
            vstack! { label, param.input, param.slider }
                .flex_1()
                .flex_basis(relative(0.28))
                .min_w(min_width)
                .gap_2()
        };
        shell(look, self.title, self.subtitle, self.action)
            .child(hstack! {}.flex_wrap().gap(metrics.section_gap).children(self.params.into_iter().map(column)))
            .children(self.footer)
    }
}

impl From<ParametersPanel> for Panel {
    fn from(panel: ParametersPanel) -> Self {
        Self::Parameters(panel)
    }
}

struct PreviewSwatch {
    visual: AnyElement,
    readout: Option<(UiCopy, UiCopy)>,
}

/// Key/value pairs shown as a readout line.
type Pairs = Vec<(UiCopy, UiCopy)>;

pub(crate) struct PreviewPanel {
    title: UiCopy,
    subtitle: Option<UiCopy>,
    swatches: Vec<PreviewSwatch>,
    reference: Option<(UiCopy, Pairs)>,
}

impl FluentBuilder for PreviewPanel {}

impl PreviewPanel {
    pub fn new(title: impl Into<UiCopy>) -> Self {
        Self { title: title.into(), subtitle: None, swatches: Vec::new(), reference: None }
    }

    pub fn subtitle(mut self, text: impl Into<UiCopy>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    /// One large swatch, sized full width × `preview_height` by the panel.
    pub fn swatch(mut self, swatch: impl IntoElement) -> Self {
        self.swatches.push(PreviewSwatch { visual: swatch.into_any_element(), readout: None });
        self
    }

    /// A swatch with its readout underneath, e.g. `Clip #00FF00`. Several sit side by side.
    pub fn labeled(mut self, swatch: impl IntoElement, key: impl Into<UiCopy>, value: impl Into<UiCopy>) -> Self {
        let readout = Some((key.into(), value.into()));
        self.swatches.push(PreviewSwatch { visual: swatch.into_any_element(), readout });
        self
    }

    /// A labeled swatch filled with one data color.
    pub fn labeled_color(self, color: impl Into<Hsla>, key: impl Into<UiCopy>, value: impl Into<UiCopy>) -> Self {
        self.labeled(div().size_full().rounded_md().bg(color.into()), key, value)
    }

    /// Reference line that ends the panel: a key, then one value.
    pub fn reference(self, key: impl Into<UiCopy>, value: impl Into<UiCopy>) -> Self {
        self.reference_values(key, [("", value)])
    }

    /// Reference line that ends the panel: a key, then key/value pairs (`Source x 0.2650 y 0.6900`).
    pub fn reference_values<K: Into<UiCopy>, V: Into<UiCopy>>(
        mut self,
        key: impl Into<UiCopy>,
        pairs: impl IntoIterator<Item = (K, V)>,
    ) -> Self {
        let pairs = pairs.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.reference = Some((key.into(), pairs));
        self
    }

    fn render(self, look: &Look, metrics: &LabMetrics) -> Div {
        let height = metrics.preview_height;
        let swatch_box = move |visual: AnyElement| div().w_full().h(height).rounded_md().child(visual);
        let shell = shell(look, self.title, self.subtitle, None);
        let shell = match <[PreviewSwatch; 1]>::try_from(self.swatches) {
            Ok([PreviewSwatch { visual, readout: None }]) => shell.child(swatch_box(visual)),
            Ok([swatch]) => shell.child(captioned(look, swatch, swatch_box)),
            Err(swatches) => shell.children((!swatches.is_empty()).then(|| {
                hstack! {}
                    .gap_2()
                    .children(swatches.into_iter().map(|swatch| captioned(look, swatch, swatch_box).flex_1()))
            })),
        };
        shell.children(self.reference.map(|(key, pairs)| {
            div()
                .flex()
                .items_baseline()
                .gap_2()
                .child(text(look, TextRole::Label, key))
                .child(readout(look, pairs))
        }))
    }
}

fn captioned(look: &Look, swatch: PreviewSwatch, swatch_box: impl Fn(AnyElement) -> Div) -> Div {
    vstack! {}
        .gap_2()
        .child(swatch_box(swatch.visual))
        .children(swatch.readout.map(|pair| readout(look, [pair])))
}

impl From<PreviewPanel> for Panel {
    fn from(panel: PreviewPanel) -> Self {
        Self::Preview(panel)
    }
}

/// One step of an ordered pipeline.
struct ChainStep {
    name: UiCopy,
    detail: UiCopy,
    active: bool,
}

/// Numbered pipeline steps; inactive steps are muted, and the panel counts the active ones.
pub(crate) struct ChainPanel {
    title: UiCopy,
    subtitle: Option<UiCopy>,
    steps: Vec<ChainStep>,
    note: Option<UiCopy>,
}

impl FluentBuilder for ChainPanel {}

impl ChainPanel {
    pub fn new(title: impl Into<UiCopy>) -> Self {
        Self { title: title.into(), subtitle: None, steps: Vec::new(), note: None }
    }

    pub fn subtitle(mut self, text: impl Into<UiCopy>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    /// A step: its name, its value or formula (a value, so mono), and whether it changes the result.
    pub fn step(mut self, name: impl Into<UiCopy>, detail: impl Into<UiCopy>, active: bool) -> Self {
        self.steps.push(ChainStep { name: name.into(), detail: detail.into(), active });
        self
    }

    /// A sentence before the generated "N of M steps active" count.
    pub fn note(mut self, text: impl Into<UiCopy>) -> Self {
        self.note = Some(text.into());
        self
    }

    fn render(self, look: &Look) -> Div {
        let chrome = look.chrome();
        let total = self.steps.len();
        let active = self.steps.iter().filter(|step| step.active).count();
        let rows = self.steps.into_iter().enumerate().map(|(index, step)| {
            let color = if step.active {
                chrome.body_text
            } else {
                chrome.muted_text
            };
            div()
                .flex()
                .items_baseline()
                .gap_3()
                .px_3()
                .py_1p5()
                .when(index > 0, |row| row.border_t_1().border_color(chrome.border))
                .child(text(look, TextRole::Label, (index + 1).to_string()).w(px(12.)).flex_shrink_0())
                .child(text(look, TextRole::Detail, step.name).flex_1().min_w_0().truncate().text_color(color))
                .child(text(look, TextRole::Value, step.detail).flex_shrink_0().text_color(color))
        });
        let count = UiCopy::deferred(move |cx| {
            crate::l10n::messages::msg_lab_chain_status(
                crate::l10n::number(&active.to_string(), cx).to_string(),
                total as f64,
                crate::l10n::number(&total.to_string(), cx).to_string(),
            )
        });
        let footer = match self.note {
            Some(note) => UiCopy::Joined(vec![note, count]),
            None => count,
        };
        shell(look, self.title, self.subtitle, None)
            .child(div().flex().flex_col().rounded_md().border_1().border_color(chrome.border).children(rows))
            .child(text(look, TextRole::Caption, footer))
    }
}

impl From<ChainPanel> for Panel {
    fn from(panel: ChainPanel) -> Self {
        Self::Chain(panel)
    }
}

/// A panel with the standard heading and a free body, for content no other panel covers.
pub(crate) struct PanelShell {
    title: UiCopy,
    subtitle: Option<UiCopy>,
    action: Option<AnyElement>,
    body: Vec<AnyElement>,
    footer: Option<UiCopy>,
}

impl FluentBuilder for PanelShell {}

impl PanelShell {
    pub fn new(title: impl Into<UiCopy>) -> Self {
        Self { title: title.into(), subtitle: None, action: None, body: Vec::new(), footer: None }
    }

    pub fn subtitle(mut self, text: impl Into<UiCopy>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    /// A control at the right of the heading.
    pub fn action(mut self, element: impl IntoElement) -> Self {
        self.action = Some(element.into_any_element());
        self
    }

    /// Append to the body; call repeatedly.
    pub fn child(mut self, element: impl IntoElement) -> Self {
        self.body.push(element.into_any_element());
        self
    }

    /// A muted closing line.
    pub fn footer(mut self, text: impl Into<UiCopy>) -> Self {
        self.footer = Some(text.into());
        self
    }

    fn render(self, look: &Look) -> Div {
        shell(look, self.title, self.subtitle, self.action)
            .children(self.body)
            .children(self.footer.map(|footer| text(look, TextRole::Caption, footer)))
    }
}

impl From<PanelShell> for Panel {
    fn from(panel: PanelShell) -> Self {
        Self::Custom(panel)
    }
}

/// A properties list: keys in `Label` in a fixed column, values in `Value` (mono), wrapping.
pub(crate) fn properties<K: Into<UiCopy>, V: Into<UiCopy>>(look: &Look, rows: impl IntoIterator<Item = (K, V)>) -> Div {
    let key_width = LabMetrics::STANDARD.property_key_width;
    div().flex().flex_col().gap_1().children(rows.into_iter().map(|(key, value)| {
        // Keys align with a value's first line, also when the value wraps.
        crate::l10n::row(
            div()
                .flex()
                .items_start()
                .gap_3()
                .child(text(look, TextRole::Label, key).w(key_width).flex_shrink_0())
                .child(text(look, TextRole::Value, value).flex_1().min_w_0()),
        )
    }))
}

/// One row of a form group: a label, its control, and whether the row is set (e.g. overridden).
pub(crate) struct FormRow {
    label: UiCopy,
    control: AnyElement,
    set: bool,
}

impl FormRow {
    pub fn new(label: impl Into<UiCopy>, control: impl IntoElement, set: bool) -> Self {
        Self { label: label.into(), control: control.into_any_element(), set }
    }
}

/// A titled column of labeled controls with aligned labels. Rows that are not `set` (still
/// following a default) show a muted label; set rows show it in foreground.
/// Groups placed in one wrapping row read as a compact form.
pub(crate) fn form_group(look: &Look, title: impl Into<UiCopy>, rows: impl IntoIterator<Item = FormRow>) -> Div {
    let metrics = LabMetrics::STANDARD;
    let chrome = look.chrome();
    vstack! {}
        .flex_1()
        .flex_basis(px(0.))
        .min_w(metrics.form_group_min_width)
        .gap_2()
        .child(text(look, TextRole::Label, title))
        .children(rows.into_iter().map(|row| {
            let color = if row.set { chrome.body_text } else { chrome.muted_text };
            crate::l10n::row(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        text(look, TextRole::Field, row.label)
                            .w(metrics.form_label_width)
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .text_color(color),
                    )
                    .child(row.control),
            )
        }))
}

/// Form groups side by side, wrapping below each other when narrow.
pub(crate) fn form_groups(groups: impl IntoIterator<Item = Div>) -> Div {
    div().flex().flex_wrap().gap(LabMetrics::STANDARD.section_gap).children(groups)
}

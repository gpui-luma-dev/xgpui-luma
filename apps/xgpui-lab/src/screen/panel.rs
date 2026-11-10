//! Supporting panels for the two-panel row.
//!
//! `ChainPanel` (Image Filters) and a free-body `PanelShell` are added with the
//! screens that need them; see `docs/lab-screen-api.md`.
use gpui::{AnyElement, Div, Hsla, IntoElement, ParentElement, SharedString, Styled, div};
use xgpui_luma::{hstack, vstack};
use xgpui_luma_look_radix::Look;

use super::{LabMetrics, TextRole, panel_surface, readout, text};

pub(crate) enum Panel {
    Parameters(ParametersPanel),
    Preview(PreviewPanel),
}

impl Panel {
    pub(super) fn render(self, look: &Look, metrics: &LabMetrics) -> Div {
        match self {
            Self::Parameters(panel) => panel.render(look, metrics),
            Self::Preview(panel) => panel.render(look, metrics),
        }
    }
}

/// Panel surface with its heading: title and subtitle stack with no gap between them.
fn shell(look: &Look, title: SharedString, subtitle: Option<SharedString>) -> Div {
    let heading = div()
        .flex()
        .flex_col()
        .child(text(look, TextRole::Section, title))
        .children(subtitle.map(|subtitle| text(look, TextRole::Caption, subtitle)));
    panel_surface(look).child(heading)
}

/// One labeled parameter: a text input above its slider.
pub(crate) struct Param {
    label: SharedString,
    input: AnyElement,
    slider: AnyElement,
}

impl Param {
    pub fn new(label: impl Into<SharedString>, input: impl IntoElement, slider: impl IntoElement) -> Self {
        Self { label: label.into(), input: input.into_any_element(), slider: slider.into_any_element() }
    }
}

pub(crate) struct ParametersPanel {
    title: SharedString,
    subtitle: Option<SharedString>,
    params: Vec<Param>,
    footer: Option<AnyElement>,
}

impl ParametersPanel {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self { title: title.into(), subtitle: None, params: Vec::new(), footer: None }
    }

    pub fn subtitle(mut self, text: impl Into<SharedString>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    /// Parameters in a wrapping row of equal columns.
    pub fn params(mut self, params: impl IntoIterator<Item = Param>) -> Self {
        self.params = params.into_iter().collect();
        self
    }

    /// A row under the parameters, e.g. preset buttons.
    pub fn footer(mut self, element: impl IntoElement) -> Self {
        self.footer = Some(element.into_any_element());
        self
    }

    fn render(self, look: &Look, metrics: &LabMetrics) -> Div {
        let min_width = metrics.param_min_width;
        shell(look, self.title, self.subtitle)
            .child(hstack! {}.flex_wrap().gap(metrics.section_gap).children(self.params.into_iter().map(
                move |param| vstack! { param.label, param.input, param.slider }.flex_1().min_w(min_width).gap_2(),
            )))
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
    readout: Option<(SharedString, SharedString)>,
}

/// Key/value pairs shown as a readout line.
type Pairs = Vec<(SharedString, SharedString)>;

pub(crate) struct PreviewPanel {
    title: SharedString,
    subtitle: Option<SharedString>,
    swatches: Vec<PreviewSwatch>,
    reference: Option<(SharedString, Pairs)>,
}

impl PreviewPanel {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self { title: title.into(), subtitle: None, swatches: Vec::new(), reference: None }
    }

    pub fn subtitle(mut self, text: impl Into<SharedString>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    /// One large swatch, sized full width × `preview_height` by the panel.
    pub fn swatch(mut self, swatch: impl IntoElement) -> Self {
        self.swatches.push(PreviewSwatch { visual: swatch.into_any_element(), readout: None });
        self
    }

    /// A swatch with its readout underneath, e.g. `Clip #00FF00`. Several sit side by side.
    pub fn labeled(
        mut self,
        swatch: impl IntoElement,
        key: impl Into<SharedString>,
        value: impl Into<SharedString>,
    ) -> Self {
        let readout = Some((key.into(), value.into()));
        self.swatches.push(PreviewSwatch { visual: swatch.into_any_element(), readout });
        self
    }

    /// A labeled swatch filled with one data color.
    pub fn labeled_color(
        self,
        color: impl Into<Hsla>,
        key: impl Into<SharedString>,
        value: impl Into<SharedString>,
    ) -> Self {
        self.labeled(div().size_full().rounded_md().bg(color.into()), key, value)
    }

    /// Reference line that ends the panel: a key, then one value.
    pub fn reference(self, key: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.reference_values(key, [("", value)])
    }

    /// Reference line that ends the panel: a key, then key/value pairs (`Source x 0.2650 y 0.6900`).
    pub fn reference_values<K: Into<SharedString>, V: Into<SharedString>>(
        mut self,
        key: impl Into<SharedString>,
        pairs: impl IntoIterator<Item = (K, V)>,
    ) -> Self {
        let pairs = pairs.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.reference = Some((key.into(), pairs));
        self
    }

    fn render(self, look: &Look, metrics: &LabMetrics) -> Div {
        let height = metrics.preview_height;
        let swatch_box = move |visual: AnyElement| div().w_full().h(height).rounded_md().child(visual);
        let shell = shell(look, self.title, self.subtitle);
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

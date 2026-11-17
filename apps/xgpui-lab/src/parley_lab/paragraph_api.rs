//! GPUI's paragraph API on the application's own text system (xgpui#325). Unlike
//! the Paragraphs tab, which drives an isolated Parley engine through the
//! extension, this page calls only `window.text_system()`, so the same code runs
//! on native text (left-to-right fallback) and under `--text-system=parley`.
use std::sync::Arc;

use gpui::{
    Bounds, Context, FocusHandle, Focusable, Font, FontFallbacks, FontRun, HighlightStyle, KeyDownEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, ParagraphPosition, ParagraphShapeOptions, ParagraphSupport, Pixels,
    PlatformParagraphLayout, Point, Render, SharedString, StrikethroughStyle, StyledText, TextAffinity, TextDirection,
    UnderlineStyle, VisualDirection, Window, canvas, div, fill, hsla, paint_paragraph, prelude::*, px, rgb,
};
use xgpui_luma_look_radix::Look;

use super::palette;
use crate::{
    lab_controls::Segmented,
    screen::{Hero, LabScreen, PanelShell, TextRole, labeled_control, properties, text},
};

const SAMPLES: [(&str, &str); 3] = [
    (
        "Latin",
        "Paragraphs wrap, hit test and select through GPUI on every text system. Resize the window to reflow.",
    ),
    (
        "Hebrew",
        "פסקה בעברית שעוברת שבירת שורות, בדיקת פגיעה ובחירה דרך GPUI. שנו את גודל החלון כדי לראות זרימה מחדש.",
    ),
    (
        "Mixed",
        "Mixed text: שלום עולם inside a Latin sentence, then مرحبا بالعالم, then the numbers 2026 and 42.",
    ),
];

const DIRECTIONS: [(&str, TextDirection); 3] = [
    ("Auto", TextDirection::Auto),
    ("Left to right", TextDirection::LeftToRight),
    ("Right to left", TextDirection::RightToLeft),
];

/// Fit the panel, or the paragraph's own min-content / max-content width.
const WIDTHS: [&str; 3] = ["Fit", "Min-content", "Max-content"];

/// Plain GPUI text: single-line labels through the flat-line path (xgpui#325 B).
const LABELS: [(&str, &str); 5] = [
    ("Latin", "Plain GPUI text label"),
    ("Hebrew", "עברית"),
    ("Arabic", "مرحبا بالعالم"),
    ("Mixed", "abc עברית 123"),
    ("Punctuation", "שלום, world!"),
];

/// Three Hebrew words with different per-word styles (xgpui#325 C).
const STYLED: &str = "שלום עולם יפה";

const FONT_SIZE: Pixels = px(22.0);
const PREVIEW_PADDING: Pixels = px(12.0);

fn font() -> Font {
    let mut font = gpui::font("IBM Plex Sans");
    font.fallbacks = Some(FontFallbacks::from_fonts(vec!["Noto Sans Hebrew".into(), "Noto Sans Arabic".into()]));
    font
}

fn start() -> ParagraphPosition {
    ParagraphPosition { byte_index: 0, affinity: TextAffinity::Downstream }
}

struct TypesetInstalled;
impl gpui::Global for TypesetInstalled {}

pub(crate) fn typeset_active(cx: &gpui::App) -> bool {
    cx.try_global::<TypesetInstalled>().is_some()
}

/// Typeset must share the application's backend: FontIds belong to that instance.
pub(crate) fn install_typeset(backend: Arc<xgpui_parley::ParleyTextSystem>, cx: &mut gpui::App) {
    xgpui_parley::typeset::install(backend, cx);
    cx.set_global(TypesetInstalled);
}

pub struct ParagraphApiPage {
    look: Arc<Look>,
    tooltip_buttons: Vec<gpui::Entity<xgpui_luma::controls::button::Button>>,
    typeset_active: bool,
    sample: usize,
    direction: usize,
    width: usize,
    sample_choice: Segmented,
    direction_choice: Segmented,
    width_choice: Segmented,
    focus: FocusHandle,
    anchor: ParagraphPosition,
    caret: ParagraphPosition,
    dragging: bool,
    geometry: Option<(Bounds<Pixels>, Arc<dyn PlatformParagraphLayout>)>,
}

/// One frame's paragraph and the numbers the readout shows.
struct Laid {
    layout: Arc<dyn PlatformParagraphLayout>,
    width: Pixels,
    content: (Pixels, Pixels),
}

impl ParagraphApiPage {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let _ = cx.text_system().add_fonts(super::paragraph::FONTS.iter().map(|data| (*data).into()).collect());
        let sample_choice = Segmented::new(
            "paragraph-api-sample",
            "Sample",
            &look,
            &SAMPLES.map(|(label, _)| label),
            0,
            cx,
            |this: &mut Self, index, cx| {
                this.sample = index;
                this.reset(cx);
            },
        );
        let direction_choice = Segmented::new(
            "paragraph-api-direction",
            "Direction",
            &look,
            &DIRECTIONS.map(|(label, _)| label),
            0,
            cx,
            |this: &mut Self, index, cx| {
                this.direction = index;
                this.reset(cx);
            },
        );
        let width_choice =
            Segmented::new("paragraph-api-width", "Width", &look, &WIDTHS, 0, cx, |this: &mut Self, index, cx| {
                this.width = index;
                this.reset(cx);
            });
        let typeset_active = cx.try_global::<TypesetInstalled>().is_some();
        let tooltip_buttons = examples::tooltip_buttons(&look, cx);
        Self {
            look,
            tooltip_buttons,
            typeset_active,
            sample: 0,
            direction: 0,
            width: 0,
            sample_choice,
            direction_choice,
            width_choice,
            focus: cx.focus_handle(),
            anchor: start(),
            caret: start(),
            dragging: false,
            geometry: None,
        }
    }

    fn reset(&mut self, cx: &mut Context<Self>) {
        self.anchor = start();
        self.caret = start();
        self.dragging = false;
        self.geometry = None;
        cx.notify();
    }

    fn lay_out(&self, window: &Window) -> anyhow::Result<Laid> {
        let system = window.text_system();
        let text = SAMPLES[self.sample].1;
        let runs = [FontRun { len: text.len(), font_id: system.resolve_font(&font()) }];
        let options = ParagraphShapeOptions::default().with_direction(DIRECTIONS[self.direction].1);
        let content = system.paragraph_content_widths_with_options(text.into(), FONT_SIZE, &runs, &options)?;
        let fit = (window.viewport_size().width - px(420.0)).clamp(px(120.0), px(640.0));
        let width = match self.width {
            1 => content.0,
            2 => content.1,
            _ => fit,
        };
        let layout = system.layout_paragraph_with_options(text.into(), FONT_SIZE, &runs, &options, width)?;
        Ok(Laid { layout, width, content })
    }

    fn select_point(&mut self, position: Point<Pixels>) {
        if let Some((bounds, layout)) = &self.geometry
            && let Ok(hit) = layout.hit_test(position - bounds.origin)
        {
            self.caret = hit;
        }
    }

    fn key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let modifiers = event.keystroke.modifiers;
        if modifiers.platform || modifiers.control || modifiers.alt {
            return;
        }
        let direction = match event.keystroke.key.as_str() {
            "left" => VisualDirection::Left,
            "right" => VisualDirection::Right,
            _ => return,
        };
        if let Some((_, layout)) = &self.geometry
            && let Ok(next) = layout.move_visual(self.caret, direction)
        {
            self.caret = next;
            if !modifiers.shift {
                self.anchor = next;
            }
            cx.stop_propagation();
            cx.notify();
        }
    }

    /// The canvas has no text nodes, so expose its source on the focus target.
    fn accessible_preview(&self, cx: &gpui::App) -> gpui::Stateful<gpui::Div> {
        div()
            .id("paragraph-api-surface")
            .role(gpui::Role::MultilineTextInput)
            .aria_label(crate::l10n::resolve(&format!("{} paragraph preview", SAMPLES[self.sample].0), cx))
            .aria_value(SAMPLES[self.sample].1)
            .aria_description(crate::l10n::resolve(
                "Read-only paragraph. Left and Right move the caret visually; hold Shift to extend the selection.",
                cx,
            ))
            .aria_keyshortcuts("ArrowLeft ArrowRight Shift+ArrowLeft Shift+ArrowRight")
            .a11y_synthetic_children(|builder| builder.parent_node().set_read_only())
    }

    fn plain_text(&self) -> gpui::Div {
        let red = hsla(0.0, 0.75, 0.5, 1.0);
        let green = hsla(0.33, 0.6, 0.75, 1.0);
        let blue = hsla(0.6, 0.8, 0.5, 1.0);
        let mut highlights = Vec::new();
        let mut offset = 0;
        for (word, style) in STYLED.split_inclusive(' ').zip([
            HighlightStyle {
                color: Some(red),
                underline: Some(UnderlineStyle { color: Some(red), thickness: px(2.0), wavy: false }),
                ..Default::default()
            },
            HighlightStyle { background_color: Some(green), ..Default::default() },
            HighlightStyle {
                color: Some(blue),
                strikethrough: Some(StrikethroughStyle { color: Some(blue), thickness: px(2.0) }),
                ..Default::default()
            },
        ]) {
            highlights.push((offset..offset + word.trim_end().len(), style));
            offset += word.len();
        }
        div()
            .flex()
            .flex_col()
            .gap_2()
            .children(LABELS.iter().map(|(name, label)| {
                div()
                    .flex()
                    .items_baseline()
                    .gap_3()
                    .child(div().w(px(110.0)).child(text(&self.look, TextRole::Caption, *name)))
                    .child(div().font(font()).text_size(FONT_SIZE).whitespace_nowrap().child(*label))
            }))
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap_3()
                    .child(div().w(px(110.0)).child(text(&self.look, TextRole::Caption, "Styled runs")))
                    .child(
                        div()
                            .font(font())
                            .text_size(FONT_SIZE)
                            .whitespace_nowrap()
                            .child(StyledText::new(SharedString::from(STYLED)).with_highlights(highlights)),
                    ),
            )
    }
}

impl Focusable for ParagraphApiPage {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for ParagraphApiPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let support = window.text_system().paragraph_support();
        let support_note = match support {
            ParagraphSupport::Full => "bidirectional; direction is honored",
            ParagraphSupport::LeftToRight => "native fallback; direction sets each line's edge, no paragraph bidi",
            _ => "unsupported; paragraph layout returns an error",
        };
        let backend = if self.typeset_active { "Parley" } else { "Native" };
        let mut page = LabScreen::new("parley-paragraph-api", self.look.clone())
            .crumb("xgpui-parley", "Paragraph API")
            .title(format!("Paragraph API · {backend}"))
            .lede(if self.typeset_active {
                "Running on the Parley text backend. Full bidirectional paragraph layout is enabled."
            } else {
                "Running on the native text backend. Paragraphs use the native fallback; mixed text has no full paragraph bidi reordering."
            })
            .caption(format!("{backend} backend · paragraph_support(): {support:?} · {support_note}"))
            .control(labeled_control(&self.look, self.sample_choice.clone().into_element()))
            .control(labeled_control(&self.look, self.direction_choice.clone().into_element()))
            .control(labeled_control(&self.look, self.width_choice.clone().into_element()));

        let readout = match self.lay_out(window) {
            Err(error) => {
                page = page.notice(Some(format!("Paragraph layout failed: {error:#}")));
                vec![("Layout".to_owned(), "failed".to_owned())]
            }
            Ok(laid) => {
                let size = laid.layout.size();
                let entity = cx.entity();
                let layout = laid.layout.clone();
                let (anchor, caret) = (self.anchor, self.caret);
                let focused = self.focus.is_focused(window);
                page = page.hero(Hero::full(
                    self.accessible_preview(cx)
                        .track_focus(&self.focus)
                        .on_key_down(cx.listener(Self::key))
                        // Keep the paper's edge at the paragraph's edge. A stretched
                        // surface makes RTL text look left-aligned in a wider field.
                        .w(size.width + PREVIEW_PADDING * 2.0)
                        .p(PREVIEW_PADDING)
                        .debug_selector(|| "paragraph-api-paper".to_owned())
                        .bg(rgb(palette::PAPER))
                        .flex_none()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, event: &MouseDownEvent, window, cx| {
                                window.focus(&this.focus, cx);
                                this.select_point(event.position);
                                if !event.modifiers.shift {
                                    this.anchor = this.caret;
                                }
                                this.dragging = true;
                                cx.notify();
                            }),
                        )
                        .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                            if this.dragging {
                                this.select_point(event.position);
                                cx.notify();
                            }
                        }))
                        .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, _| this.dragging = false))
                        .on_mouse_up_out(MouseButton::Left, cx.listener(|this, _, _, _| this.dragging = false))
                        .child(
                            canvas(
                                move |bounds, _, cx| {
                                    entity.update(cx, |this, _| this.geometry = Some((bounds, layout.clone())));
                                    layout
                                },
                                move |bounds, layout, window, _| {
                                    if let Ok(rects) = layout.selection_bounds(anchor, caret) {
                                        for rect in rects {
                                            window.paint_quad(fill(
                                                Bounds::new(bounds.origin + rect.origin, rect.size),
                                                rgb(palette::SELECTION),
                                            ));
                                        }
                                    }
                                    if let Err(error) = paint_paragraph(
                                        layout.as_ref(),
                                        bounds.origin,
                                        |_| rgb(palette::INK).into(),
                                        window,
                                    ) {
                                        eprintln!("Paragraph paint: {error:#}");
                                    }
                                    if focused && let Ok(rect) = layout.caret_bounds(caret) {
                                        window.paint_quad(fill(
                                            Bounds::new(bounds.origin + rect.origin, rect.size),
                                            rgb(palette::CARET),
                                        ));
                                    }
                                },
                            )
                            .w(size.width)
                            .h(size.height),
                        ),
                ));
                vec![
                    ("Min-content".to_owned(), format!("{:.1} px", f32::from(laid.content.0))),
                    ("Max-content".to_owned(), format!("{:.1} px", f32::from(laid.content.1))),
                    (
                        "Laid out".to_owned(),
                        format!(
                            "{:.1} px wide → {:.1} × {:.1} px",
                            f32::from(laid.width),
                            f32::from(size.width),
                            f32::from(size.height)
                        ),
                    ),
                    ("Caret".to_owned(), format!("{} {:?}", self.caret.byte_index, self.caret.affinity)),
                    ("Anchor".to_owned(), format!("{} {:?}", self.anchor.byte_index, self.anchor.affinity)),
                ]
            }
        };
        page.panels(
            PanelShell::new("Measurements / live").child(properties(&self.look, readout)),
            PanelShell::new("Plain GPUI text").child(self.plain_text()),
        )
        .section_panel(PanelShell::new("End truncation").child(examples::truncation(&self.look)))
        .section_panel(PanelShell::new("Wrapped RTL: plain GPUI / typeset()").child(examples::wrapped(
            &self.look,
            self.typeset_active,
            &self.tooltip_buttons,
        )))
        .note(
            "Click or drag to select; Left/Right move visually and Shift extends. On Parley, Hebrew and the Right to \
             left direction start at the right edge on both text systems; only Parley also reorders mixed text \
             across the paragraph. Plain text \
             labels should read right to left on both, and the styled Hebrew line should be red and underlined on the \
             right, green in the middle and blue and struck through on the left.",
        )
        .into_any_element()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{TestAppContext, TestDispatcher, VisualTestContext, point, size};

    /// Where the paragraph's last (shortest) line sits.
    #[derive(Debug, PartialEq)]
    enum Edge {
        Left,
        Right,
    }

    /// Draws the page and reports the backend's support and which edge the
    /// paragraph's last line is flush with.
    fn draw(parley: bool, sample: usize, direction: usize) -> (ParagraphSupport, Edge) {
        let mut cx = if parley {
            let backend = crate::parley_lab::Engine::new().unwrap().backend.clone();
            TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend)
        } else {
            TestAppContext::single()
        };
        cx.update(|cx| xgpui_luma::init(cx).unwrap());
        let look = crate::theme::default_look();
        let window = cx.open_window(size(px(1000.0), px(900.0)), move |_, cx| ParagraphApiPage::new(look, cx));
        let page = window.root(&mut cx).unwrap();
        window
            .update(&mut cx, |page, _, cx| {
                page.sample_choice.select(sample, cx);
                page.direction_choice.select(direction, cx);
            })
            .unwrap();
        let mut visual = VisualTestContext::from_window(window.into(), &cx);
        visual.draw(point(px(0.0), px(0.0)), size(px(1000.0), px(900.0)), move |_, _| {
            div().size_full().child(page.clone())
        });
        let paper = visual.debug_bounds("paragraph-api-paper").expect("preview paper");
        window
            .update(&mut cx, |page, window, cx| {
                use gpui::Element as _;
                let preview = page.accessible_preview(cx);
                assert_eq!(preview.a11y_role(), Some(gpui::Role::MultilineTextInput));
                let mut node = gpui::accesskit::Node::new(gpui::Role::MultilineTextInput);
                preview.write_a11y_info(&mut node);
                assert_eq!(node.label(), Some(format!("{} paragraph preview", SAMPLES[sample].0).as_str()));
                assert_eq!(node.value(), Some(SAMPLES[sample].1));
                assert!(node.description().unwrap().contains("Shift"));
                let support = window.text_system().paragraph_support();
                let (canvas, layout) = page.geometry.as_ref().expect("the paragraph laid out");
                assert!((paper.left() + PREVIEW_PADDING - canvas.left()).abs() < px(1.0));
                assert!(
                    (paper.right() - PREVIEW_PADDING - canvas.right()).abs() < px(1.0),
                    "the paper must not stretch beyond the paragraph's RTL alignment edge"
                );
                let text = SAMPLES[sample].1;
                let end = ParagraphPosition { byte_index: text.len(), affinity: TextAffinity::Upstream };
                let rects = layout.selection_bounds(start(), end).unwrap();
                let last = rects.iter().max_by(|a, b| f32::from(a.top()).total_cmp(&f32::from(b.top()))).unwrap();
                let width = layout.size().width;
                assert!(last.size.width < width - px(20.0), "the last line is shorter than the paragraph");
                let edge = if (width - last.right()).abs() < px(1.0) {
                    Edge::Right
                } else {
                    Edge::Left
                };
                (support, edge)
            })
            .unwrap()
    }

    #[test]
    fn hebrew_and_explicit_rtl_align_right_on_both_text_systems() {
        assert_eq!(draw(true, 1, 0), (ParagraphSupport::Full, Edge::Right), "Parley, Hebrew, auto");
        assert_eq!(draw(true, 0, 2), (ParagraphSupport::Full, Edge::Right), "Parley, Latin, right to left");
        assert_eq!(draw(true, 0, 0), (ParagraphSupport::Full, Edge::Left), "Parley, Latin, auto");
        assert_eq!(draw(true, 1, 1), (ParagraphSupport::Full, Edge::Left), "Parley, Hebrew, left to right");
        // The native fallback aligns to the same start edge (xgpui#325).
        assert_eq!(draw(false, 1, 0), (ParagraphSupport::LeftToRight, Edge::Right), "native, Hebrew, auto");
        assert_eq!(draw(false, 0, 2), (ParagraphSupport::LeftToRight, Edge::Right), "native, Latin, right to left");
        assert_eq!(draw(false, 0, 0), (ParagraphSupport::LeftToRight, Edge::Left), "native, Latin, auto");
        assert_eq!(draw(false, 1, 1), (ParagraphSupport::LeftToRight, Edge::Left), "native, Hebrew, left to right");
    }
}

mod examples;

#[cfg(all(test, feature = "test-support"))]
mod rtl_checks;

//! Read-only line-breaking and hit-test probe, separate from the paragraph editor.
use super::*;
use crate::screen::{Hero, LabScreen, PanelShell, ParametersPanel, Number, properties, labeled_control};
struct Snapshot {
    key: (usize, u32, u32, u32),
    layout: Arc<dyn PlatformParagraphLayout>,
    image: Arc<RenderImage>,
}
pub(super) struct LinePage {
    look: Arc<Look>,
    numbers: Vec<Number>,
    engine: Rc<Engine>,
    sample: usize,
    width: f32,
    size: f32,
    buttons: Vec<Entity<Button>>,
    sample_choice: crate::lab_controls::Choice,
    sliders: Vec<Slider>,
    snapshot: Option<Snapshot>,
    bounds: Bounds<Pixels>,
    hit: Option<ParagraphPosition>,
}
impl LinePage {
    pub fn new(engine: Rc<Engine>, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let sample_choice = crate::lab_controls::Choice::new(
            "text-line-sample",
            "Sample",
            &look,
            paragraph::TEXTS.iter().map(|(label, _)| label.to_string()).collect(),
            0,
            cx,
            |this: &mut Self, index, cx| {
                this.sample = index;
                this.hit = None;
                this.snapshot = None;
                cx.notify();
            },
        );
        let buttons = vec![
            radix::Button::new("line-reset")
                .look(look.as_ref())
                .outline()
                .label("Reset layout")
                .on_click(cx, |this: &mut Self, cx| {
                    this.width = 320.;
                    this.size = 24.;
                    for (id, slider) in this.sliders.iter().enumerate() {
                        slider.update(cx, |slider, cx| slider.set_value(if id == 0 { 320. } else { 24. }, cx));
                    }
                    this.hit = None;
                    cx.notify();
                })
                .spawn(cx),
        ];
        let sliders = [(120., 720., 320.), (12., 48., 24.)]
            .into_iter()
            .enumerate()
            .map(|(id, (min, max, value))| {
                radix::Slider::new(format!("line-value-{id}"))
                    .look(look.as_ref())
                    .range(min..max)
                    .step(1.)
                    .value(value)
                    .on_value_changed(cx, move |this: &mut Self, _, value, cx| {
                        if id == 0 {
                            this.width = value;
                        } else {
                            this.size = value;
                        }
                        this.hit = None;
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let mut page = Self {
            look,
            numbers: Vec::new(),
            engine,
            sample: 0,
            width: 320.,
            size: 24.,
            buttons,
            sample_choice,
            sliders,
            snapshot: None,
            bounds: Default::default(),
            hit: None,
        };
        page.numbers = page
            .sliders
            .iter()
            .enumerate()
            .map(|(i, slider)| {
                Number::attach(
                    &format!("line-{i}"),
                    if i == 0 { "Wrap width (px)" } else { "Font size (px)" },
                    if i == 0 { (120., 720., 1.) } else { (12., 48., 1.) },
                    if i == 0 { 320. } else { 24. },
                    slider.clone(),
                    &page.look,
                    cx,
                    move |this: &mut Self, value, _| {
                        if i == 0 {
                            this.width = value;
                        } else {
                            this.size = value;
                        }
                        this.hit = None;
                        value
                    },
                )
            })
            .collect();
        page
    }
    fn prepare(&mut self, scale: f32) -> Result<()> {
        let key = (self.sample, self.width.to_bits(), self.size.to_bits(), scale.to_bits());
        if self.snapshot.as_ref().is_some_and(|snapshot| snapshot.key == key) {
            return Ok(());
        }
        let source = paragraph::TEXTS[self.sample].1;
        let run = FontRun { len: source.len(), font_id: self.engine.system.resolve_font(&paragraph::font()) };
        let layout = self.engine.cache.layout_paragraph(source.into(), px(self.size), &[run], px(self.width))?;
        let image = raster::render(&*self.engine.backend, layout.runs(), px(self.size), layout.size(), scale, 0.)?;
        self.snapshot = Some(Snapshot { key, layout, image });
        Ok(())
    }
}
impl Render for LinePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Err(error) = self.prepare(window.scale_factor()) {
            return div().child(format!("Line layout unavailable: {error:#}")).into_any_element();
        }
        let snapshot = self.snapshot.as_ref().unwrap();
        let layout = snapshot.layout.clone();
        let image = snapshot.image.clone();
        let owner = cx.entity();
        let hit = self.hit;
        let mut rows = std::collections::BTreeMap::<u32, Vec<usize>>::new();
        for byte in layout.source().char_indices().map(|(byte, _)| byte).chain(std::iter::once(layout.source().len())) {
            if let Ok(caret) =
                layout.caret_bounds(ParagraphPosition { byte_index: byte, affinity: TextAffinity::Downstream })
            {
                rows.entry(f32::from(caret.top()).to_bits()).or_default().push(byte);
            }
        }
        let height = layout.size().height;
        let run_count = layout.runs().len();
        self.numbers[0].observe(self.width, cx);
        self.numbers[1].observe(self.size, cx);
        let surface = div().id("line-surface-scroll").w_full().overflow_x_scroll().child(
            div()
                .id("line-hit-surface")
                .w(px(self.width))
                .h(height)
                .flex_shrink_0()
                .bg(rgb(palette::PAPER))
                .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                    this.hit = this
                        .snapshot
                        .as_ref()
                        .and_then(|snapshot| snapshot.layout.hit_test(event.position - this.bounds.origin).ok());
                    cx.notify();
                }))
                .child(
                    canvas(
                        move |bounds, _, cx| {
                            owner.update(cx, |this, _| this.bounds = bounds);
                        },
                        move |bounds, _, window, _| {
                            let _ = window.paint_image(bounds, bounds, Corners::default(), image.clone(), 0, false);
                            if let Some(hit) = hit
                                && let Ok(caret) = layout.caret_bounds(hit)
                            {
                                window.paint_quad(fill(
                                    Bounds::new(bounds.origin + caret.origin, caret.size),
                                    rgb(palette::CARET),
                                ));
                            }
                        },
                    )
                    .size_full(),
                ),
        );
        let mut details = PanelShell::new("Line geometry").child(properties(
            &self.look,
            [
                ("Visual rows", rows.len().to_string()),
                ("Height", format!("{:.1} px", f32::from(height))),
                ("Resolved runs", run_count.to_string()),
                ("Hit", format!("{:?}", self.hit)),
            ],
        ));
        for (y, bytes) in rows {
            details = details.child(crate::screen::text(
                &self.look,
                crate::screen::TextRole::Value,
                format!("y={:.1} px · byte boundaries {:?}", f32::from_bits(y), bytes),
            ));
        }
        let mut root = LabScreen::new("parley-line", self.look.clone()).crumb("xgpui-parley", "Line Layout").title("Line Layout")
            .lede("Explore how text wraps and how the pointer finds a character boundary.")
            .hero(Hero::full(surface)).caption(format!("{} · {:.0} px wrap · {:.0} px text", paragraph::TEXTS[self.sample].0, self.width, self.size))
            .control(labeled_control(&self.look, self.sample_choice.clone().into_element()))
            .panels(ParametersPanel::new("Layout parameters").params(self.numbers.iter().map(|n|n.param())), details)
            .note("Move the pointer over the text to inspect cluster boundaries. Width changes reflow the retained source; editing and IME are available in Paragraphs.")
            .credits("Read-only CPU raster preview from the isolated Parley backend. Paper and caret colors are fixed inspection data.");
        for button in &self.buttons {
            root = root.action(button.clone());
        }
        root.into_any_element()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use xgpui_luma::controls::{button::ButtonEvent, textfield::TextFieldEvent};
    #[::core::prelude::v1::test]
    fn numeric_width_reflows_source_and_reset_restores_geometry() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) =
            app.add_window_view(|_, cx| LinePage::new(Engine::new().unwrap(), crate::theme::default_look(), cx));
        let (field, initial_height) = cx.update(|_, cx| {
            let p = page.read(cx);
            (p.numbers[0].field.clone(), p.snapshot.as_ref().unwrap().layout.size().height)
        });
        field.update(cx, |f, cx| {
            f.set_value("600", cx);
            cx.emit(TextFieldEvent::Change { value: "600".into() });
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let p = page.read(cx);
            assert_eq!(p.width, 600.);
            let layout = &p.snapshot.as_ref().unwrap().layout;
            assert_eq!(layout.source(), paragraph::TEXTS[0].1);
            assert!(layout.size().height < initial_height);
        });
        let reset = cx.update(|_, cx| page.read(cx).buttons[0].clone());
        reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(field.read(cx).value(), "320.0");
            assert_eq!(page.read(cx).snapshot.as_ref().unwrap().layout.size().height, initial_height);
        });
    }
}

//! The xgpui text-lab corpus, diagnostics and editable paragraphs in the unified shell.
mod samples;
mod timings;
mod paragraph;
mod raster;
pub(crate) mod native;
mod workload;
mod line;
mod screens;
mod palette;
pub(crate) mod text_field;

use std::{rc::Rc, sync::Arc};
use anyhow::Result;
use gpui::{prelude::*, *};
use xgpui_luma::{
    prelude::HasPresenter,
    controls::{button::Button, slider::Slider},
};
use xgpui_luma_look_radix::{self as radix, Look};
use crate::theme::ColorVizLookExt;

/// Present when the whole application runs on the Parley text system
/// (`--text-system=parley`, or the standalone `--native-text-backend=parley`
/// window). Controls that edit through GPUI's paragraph API need it.
pub(crate) struct ParleyTextSystemActive;
impl Global for ParleyTextSystemActive {}

/// The Parley text system for a whole application, with the lab's sample fonts
/// (system fonts are found through Parley's `system-fonts` feature).
pub(crate) fn application_text_system() -> Result<Arc<dyn PlatformTextSystem>> {
    Ok(Engine::new()?.backend.clone())
}

struct Engine {
    backend: Arc<xgpui_parley::ParleyTextSystem>,
    system: Arc<TextSystem>,
    cache: WindowTextSystem,
}
impl Engine {
    fn new() -> Result<Rc<Self>> {
        let mut fonts = parley::FontContext::new();
        for data in [samples::PLEX, samples::INTER, paragraph::FONTS[0], paragraph::FONTS[1]] {
            fonts.collection.register_fonts(parley::fontique::Blob::new(Arc::new(data.to_vec())), None);
        }
        let backend = Arc::new(xgpui_parley::ParleyTextSystem::new(fonts, "IBM Plex Sans")?);
        let system = Arc::new(TextSystem::new(backend.clone()));
        let cache = WindowTextSystem::new(system.clone());
        Ok(Rc::new(Self { backend, system, cache }))
    }
}
struct Row {
    label: String,
    text: String,
    font: Font,
    width: f32,
    height: f32,
    image: Arc<RenderImage>,
    details: String,
    faces: usize,
}

pub struct ParleyPage {
    pub active_tab: SharedString,
    look: Arc<Look>,
    engine: Result<Rc<Engine>>,
    paragraph: Option<Entity<paragraph::ParagraphLab>>,
    line: Option<Entity<line::LinePage>>,
    /// The Parley text field and text area, in the application's active look.
    text_field: Entity<text_field::page::TextFieldPage>,
    scroll: Entity<workload::ScrollLab>,
    children: Vec<std::process::Child>,
    child_error: Option<String>,
    scroll_report: Option<Result<String>>,
    buttons: Vec<Entity<Button>>,
    sliders: Vec<Slider>,
    numbers: Vec<crate::screen::Number>,
    font_size: f32,
    raster_scale: f32,
    rows: Option<Result<Vec<Row>>>,
    report: Option<Result<Vec<timings::Timing>>>,
    measuring: bool,
}
impl ParleyPage {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let engine = Engine::new();
        // Native reference rows use the same registered source fixtures.
        let _ = cx
            .text_system()
            .add_fonts(vec![std::borrow::Cow::Borrowed(samples::INTER), std::borrow::Cow::Borrowed(samples::PLEX)]);
        let paragraph = engine.as_ref().ok().map(|engine| {
            let timings = Vec::new();
            cx.new(|cx| paragraph::ParagraphLab::new(engine.clone(), timings, look.clone(), cx))
        });
        let line = engine
            .as_ref()
            .ok()
            .map(|engine| cx.new(|cx| line::LinePage::new(engine.clone(), look.clone(), cx)));
        let text_field = cx.new(|cx| text_field::page::TextFieldPage::new(look.clone(), false, true, cx));
        let scroll = cx.new(|cx| {
            let mut page = workload::ScrollLab::new(
                if cx.has_global::<ParleyTextSystemActive>() {
                    "Shell Parley backend"
                } else {
                    "Shell native backend"
                }
                .into(),
                workload::corpus(5000),
                samples::samples().unwrap().remove(2).font,
                0,
                false,
            );
            page.init_controls(look.clone(), cx);
            page.embedded = true;
            page
        });
        let buttons = [
            "Run CPU diagnostics",
            "Reset preview",
            "Open Parley glyph atlas",
            "Open native glyph atlas",
            "Measure scroll corpus CPU",
        ]
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            radix::Button::new(format!("parley-action-{index}"))
                .look(look.as_ref())
                .outline()
                .label(label)
                .on_click(cx, move |this: &mut Self, cx| {
                    if index == 0 {
                        this.measure(cx);
                    } else if index == 2 || index == 3 {
                        match std::env::current_exe().and_then(|exe| {
                            std::process::Command::new(exe)
                                .arg(if index == 2 {
                                    "--native-text-backend=parley"
                                } else {
                                    "--native-text-backend=native"
                                })
                                .spawn()
                        }) {
                            Ok(child) => {
                                this.children.push(child);
                                this.child_error = None;
                            }
                            Err(error) => this.child_error = Some(error.to_string()),
                        }
                    } else if index == 4 {
                        this.measure_scroll(cx);
                    } else {
                        this.font_size = 28.;
                        this.raster_scale = 2.;
                        this.rows = None;
                        for (index, slider) in this.sliders.iter().enumerate() {
                            slider.update(cx, |slider, cx| slider.set_value(if index == 0 { 28. } else { 2. }, cx));
                        }
                    }
                    cx.notify();
                })
                .spawn(cx)
        })
        .collect();
        let sliders = [false, true]
            .into_iter()
            .map(|scale| {
                radix::Slider::new(if scale {
                    "parley-raster-scale"
                } else {
                    "parley-font-size"
                })
                .look(look.as_ref())
                .size(radix::SliderSize::One)
                .range(if scale { 1.0..4.0 } else { 12.0..64.0 })
                .step(if scale { 0.25 } else { 1. })
                .value(if scale { 2. } else { 28. })
                .on_value_changed(cx, move |this: &mut Self, _, value, cx| {
                    if scale {
                        this.raster_scale = value;
                    } else {
                        this.font_size = value;
                    }
                    this.rows = None;
                    cx.notify();
                })
                .spawn(cx)
            })
            .collect();
        let mut page = Self {
            numbers: Vec::new(),
            active_tab: "Text Shaping".into(),
            look,
            engine,
            paragraph,
            line,
            text_field,
            scroll,
            children: vec![],
            child_error: None,
            scroll_report: None,
            buttons,
            sliders,
            font_size: 28.,
            raster_scale: 2.,
            rows: None,
            report: None,
            measuring: false,
        };
        page.numbers = page
            .sliders
            .iter()
            .enumerate()
            .map(|(i, slider)| {
                crate::screen::Number::attach(
                    &format!("parley-{i}"),
                    if i == 0 { "Font size (px)" } else { "Raster scale" },
                    if i == 0 { (12., 64., 1.) } else { (1., 4., 0.25) },
                    if i == 0 { 28. } else { 2. },
                    slider.clone(),
                    &page.look,
                    cx,
                    move |this: &mut Self, value, _| {
                        if i == 0 {
                            this.font_size = value;
                        } else {
                            this.raster_scale = value;
                        }
                        this.rows = None;
                        value
                    },
                )
            })
            .collect();
        page
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.scroll.update(cx, |scroll, cx| {
            scroll.suspend();
            cx.notify();
        });
    }
    pub fn activate(&mut self, tab: SharedString, cx: &mut Context<Self>) {
        self.suspend(cx);
        if let Some(paragraph) = &self.paragraph {
            paragraph.update(cx, |page, cx| {
                page.set_mode(tab == "Line Layout");
                cx.notify();
            });
        }
        self.active_tab = tab;
        cx.notify();
    }
    fn make_rows(&self) -> Result<Vec<Row>> {
        let engine = self.engine.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?;
        samples::samples()?
            .into_iter()
            .map(|sample| {
                let line = engine.cache.layout_line(&sample.text, px(self.font_size), &sample.runs(), None);
                let height = f32::from(line.ascent + line.descent).ceil() + 4.;
                let width = f32::from(line.width).ceil() + 4.;
                let image = raster::render(
                    &*engine.backend,
                    &line.runs,
                    px(self.font_size),
                    size(px(width), px(height)),
                    self.raster_scale,
                    f32::from(line.ascent) + 2.,
                )?;
                let details = line
                    .runs
                    .iter()
                    .map(|run| {
                        format!(
                            "face {:?}: {} glyphs, byte clusters {:?}",
                            run.font_id,
                            run.glyphs.len(),
                            run.glyphs.iter().map(|g| g.index).collect::<Vec<_>>()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" · ");
                let faces = line.runs.iter().map(|run| run.font_id).collect::<std::collections::HashSet<_>>().len();
                Ok(Row {
                    faces,
                    label: sample.label,
                    text: sample.text,
                    font: sample.font,
                    width,
                    height,
                    image,
                    details,
                })
            })
            .collect()
    }
    fn measure(&mut self, cx: &mut Context<Self>) {
        if self.measuring {
            return;
        }
        self.measuring = true;
        let scale = self.raster_scale;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let engine = Engine::new()?;
                    let rows = samples::samples()?;
                    let line = timings::measure(engine.system.clone(), &*engine.backend, &rows, 20, scale)?;
                    let paragraph = paragraph::measure(engine.system.clone(), 20)?;
                    Ok::<_, anyhow::Error>((line, paragraph))
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.measuring = false;
                this.report = Some(match result {
                    Ok((line, timings)) => {
                        if let Some(paragraph) = &this.paragraph {
                            paragraph.update(cx, |page, cx| {
                                page.set_timings(timings);
                                cx.notify();
                            });
                        }
                        Ok(line)
                    }
                    Err(error) => Err(error),
                });
                cx.notify();
            });
        })
        .detach();
    }
}
impl ParleyPage {
    fn measure_scroll(&mut self, cx: &mut Context<Self>) {
        if self.measuring {
            return;
        }
        self.measuring = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let engine = Engine::new()?;
                    workload::measure(
                        engine.system.clone(),
                        &workload::corpus(5000),
                        &samples::samples()?.remove(2).font,
                    )
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.measuring = false;
                this.scroll_report = Some(result);
                cx.notify();
            });
        })
        .detach();
    }
}
impl Drop for ParleyPage {
    fn drop(&mut self) {
        for child in &mut self.children {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Render for ParleyPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if self.active_tab == "Scrolling" {
            let report = self.scroll_report.as_ref().map(|report| match report {
                Ok(text) => text.clone(),
                Err(error) => format!("Scroll CPU diagnostic: {error:#}"),
            });
            self.scroll.update(_cx, |scroll, cx| {
                if scroll.lab_actions.len() != 3 || scroll.lab_report != report {
                    scroll.lab_actions = self.buttons.iter().skip(2).cloned().collect();
                    scroll.lab_report = report;
                    cx.notify();
                }
            });
            return div().size_full().child(self.scroll.clone()).into_any_element();
        }
        if self.active_tab == "Line Layout" {
            return match &self.line {
                Some(page) => div().size_full().child(page.clone()).into_any_element(),
                None => self.backend_error("parley-line", "Line Layout").into_any_element(),
            };
        }
        if self.active_tab == "Text Field" {
            return div().size_full().child(self.text_field.clone()).into_any_element();
        }
        if self.active_tab == "Paragraphs" {
            return match &self.paragraph {
                Some(page) => div().size_full().child(page.clone()).into_any_element(),
                None => self.backend_error("parley-paragraph", "Paragraphs").into_any_element(),
            };
        }
        self.children.retain_mut(|child| child.try_wait().ok().flatten().is_none());
        if self.rows.is_none() {
            self.rows = Some(self.make_rows());
        }
        self.row_screen(_cx).into_any_element()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::{ParleyPage, samples};
    use gpui::TestAppContext;
    use xgpui_luma::controls::button::ButtonEvent;

    #[test]
    fn parley_tabs_render_actual_masks_and_luma_diagnostics() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| ParleyPage::new(crate::theme::default_look(), cx));
        for tab in ["Text Shaping", "Line Layout", "Paragraphs", "Fonts & Fallback", "Rasterization"] {
            page.update(cx, |page, cx| page.activate(tab.into(), cx));
            cx.run_until_parked();
        }
        cx.update(|_, cx| {
            let page = page.read(cx);
            let rows = page.rows.as_ref().unwrap().as_ref().unwrap();
            assert_eq!(rows.len(), samples::samples().unwrap().len());
            assert!(rows[2].faces > 1, "fallback sample must actually resolve multiple faces");
            for row in rows {
                assert!(
                    row.image.as_bytes(0).unwrap().as_chunks::<4>().0.iter().any(|pixel| pixel[3] > 0),
                    "{} must contain visible glyphs",
                    row.label
                );
            }
        });
        let run = cx.update(|_, cx| page.read(cx).buttons[0].clone());
        run.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert!(!page.measuring);
            let report = page.report.as_ref().unwrap().as_ref().unwrap();
            assert_eq!(report.len(), 6);
            assert!(report.iter().all(|row| row.glyphs > 0 && row.width > 0.));
        });
    }
}

#[cfg(all(test, feature = "test-support"))]
mod screen_tests;

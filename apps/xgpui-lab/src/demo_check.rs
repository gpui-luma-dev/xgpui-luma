//! Explicit native capture probe for the imported app pages; never runs on normal navigation.
use anyhow::Result;
use gpui::{prelude::*, *};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use crate::extension_pane::ExtensionPane;
use crate::theme::ColorVizLookExt;
const PAGES: [(&str, &str); 6] = [
    ("xgpui-canvas-tabs", "Spatial Canvas"),
    ("xgpui-canvas-tabs", "Mandelbrot"),
    ("xgpui-shader-tabs", "Pastel Waves"),
    ("standalone-benchmark-tabs", "Quad Benchmark"),
    ("standalone-benchmark-tabs", "Color Field Benchmark"),
    ("standalone-benchmark-tabs", "Dense Metal Producers"),
];
struct Probe {
    panes: Vec<(&'static str, Entity<ExtensionPane>)>,
    stage: usize,
    ticks: usize,
    started: Instant,
    output: PathBuf,
    reports: Vec<serde_json::Value>,
    pending: bool,
}
impl Render for Probe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (id, page) = PAGES[self.stage];
        let pane = self.panes.iter().find(|(key, _)| *key == id).unwrap().1.clone();
        if self.ticks == 0 {
            pane.update(cx, |pane, cx| pane.activate_page(page, cx));
            self.started = Instant::now();
            let _ = window.set_color_mode(if page == "Pastel Waves" {
                WindowColorMode::Default
            } else {
                WindowColorMode::WideGamutSdr
            });
        }
        self.ticks += 1;
        if !self.pending {
            self.pending = true;
            let stage = self.stage;
            let entity = cx.entity();
            window.on_next_frame(move|window,cx|entity.update(cx,|this,cx|{
                this.pending=false;
                if (this.ticks<20 || this.started.elapsed()<Duration::from_millis(400) || !this.panes.iter().find(|(key,_)|*key==id).unwrap().1.read(cx).ready_for_capture(cx)) && this.started.elapsed()<Duration::from_secs(15) {cx.notify();window.refresh();return;}
                let result=(||->Result<()> {
                    anyhow::ensure!(this.panes.iter().find(|(key,_)|*key==id).unwrap().1.read(cx).ready_for_capture(cx),"page did not settle within 15 seconds: {page}");
                    let image=window.render_to_image()?;
                    anyhow::ensure!(image.width()>0 && image.height()>0,"empty page capture");
                    let filename=format!("{stage}.png");image.save(this.output.join(&filename))?;
                    this.reports.push(serde_json::json!({"page":page,"capture":filename,"width":image.width(),"height":image.height()}));
                    for (_,pane) in &this.panes {pane.update(cx,|pane,cx|pane.deactivate(cx));}
                    if stage+1==PAGES.len() {
                        std::fs::write(this.output.join("report.json"),serde_json::to_vec_pretty(&serde_json::json!({"pages":this.reports,"errors":[],"scope":"Native app page rendering; benchmark runs are separate"}))?)?;
                        cx.quit();
                    } else {this.stage+=1;this.ticks=0;cx.notify();window.refresh();}
                    Ok(())
                })();
                if let Err(error)=result {eprintln!("Demo check: {error:#}");std::process::exit(1);}
            }));
            window.refresh();
        }
        let chrome = crate::theme::default_look().chrome();
        div().bg(chrome.panel_background).text_color(chrome.body_text).size_full().child(pane)
    }
}
pub fn run(cx: &mut App, output: PathBuf) -> Result<()> {
    std::fs::create_dir_all(&output)?;
    xgpui_luma::init(cx)?;
    xgpui_luma::focus::bind_default_focus_keys(cx);
    xgpui_luma::key_handling::bind_default_control_keys(cx);
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1100.), px(800.)), cx))),
            inactive_frame_interval: None,
            ..Default::default()
        },
        |_, cx| {
            cx.new(|cx| {
                let look = crate::theme::default_look();
                let panes = ["xgpui-canvas-tabs", "xgpui-shader-tabs", "standalone-benchmark-tabs"]
                    .into_iter()
                    .map(|id| (id, cx.new(|cx| ExtensionPane::new(id, look.clone(), None, cx))))
                    .collect();
                Probe {
                    panes,
                    stage: 0,
                    ticks: 0,
                    started: Instant::now(),
                    output,
                    reports: Vec::new(),
                    pending: false,
                }
            })
        },
    )?;
    cx.activate(true);
    Ok(())
}

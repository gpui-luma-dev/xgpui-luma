//! Embedded standalone workloads; dense producers use a bounded copy of this executable.
mod quad;
mod field;
mod dense;
use anyhow::{Result, ensure};
use gpui::{prelude::*, *};
use xgpui_luma::{
    controls::{button::Button, slider::Slider},
    prelude::HasPresenter,
};
use xgpui_luma_look_radix::{self as radix, Look};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use crate::theme::ColorVizLookExt;
use crate::lab_controls::{Choice, labels};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Quad,
    Field,
    Dense,
}
impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Quad => "Quad Composition",
            Self::Field => "Color Field",
            Self::Dense => "Dense Metal Producers",
        }
    }
    fn slug(self) -> &'static str {
        match self {
            Self::Quad => "quad",
            Self::Field => "field",
            Self::Dense => "dense",
        }
    }
}

pub struct Page {
    look: Arc<Look>,
    kind: Kind,
    buttons: Vec<Entity<Button>>,
    count: Choice,
    pattern: Choice,
    method: Choice,
    samples: Choice,
    animation: Choice,
    backend: Choice,
    frames: Choice,
    producers: Choice,
    opacity: Slider,
    hue: Slider,
    count_index: usize,
    pattern_index: usize,
    method_index: usize,
    samples_index: usize,
    animation_index: usize,
    backend_index: usize,
    frames_index: usize,
    producers_index: usize,
    opacity_value: f32,
    hue_value: f32,
    busy: bool,
    cancel: Arc<AtomicBool>,
    status: String,
    output: Option<PathBuf>,
    report: Option<Value>,
}
impl Page {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let count = Choice::new(
            "quad-count",
            "Quads",
            &look,
            labels(&["100", "1,000", "5,000", "20,000"]),
            1,
            cx,
            |this: &mut Self, i, cx| {
                this.count_index = i;
                cx.notify();
            },
        );
        let pattern = Choice::new(
            "quad-pattern",
            "Pattern",
            &look,
            labels(&["Grid", "Overlap"]),
            0,
            cx,
            |this: &mut Self, i, cx| {
                this.pattern_index = i;
                cx.notify();
            },
        );
        let method = Choice::new(
            "field-method",
            "Method",
            &look,
            labels(&["Layered", "Sampled"]),
            0,
            cx,
            |this: &mut Self, i, cx| {
                this.method_index = i;
                cx.notify();
            },
        );
        let samples = Choice::new(
            "field-samples",
            "Sampling target",
            &look,
            labels(&["16", "64", "128", "256", "512"]),
            2,
            cx,
            |this: &mut Self, i, cx| {
                this.samples_index = i;
                cx.notify();
            },
        );
        let animation = Choice::new(
            "field-animation",
            "Benchmark hue",
            &look,
            labels(&["Static", "Advance 1° / frame"]),
            0,
            cx,
            |this: &mut Self, i, cx| {
                this.animation_index = i;
                cx.notify();
            },
        );
        let backend = Choice::new(
            "standalone-backend",
            "Measurement",
            &look,
            labels(&["Native render + readback", "CPU composition only"]),
            0,
            cx,
            |this: &mut Self, i, cx| {
                this.backend_index = i;
                cx.notify();
            },
        );
        let frames = Choice::new(
            "standalone-frames",
            "Measured frames",
            &look,
            labels(&["5", "60", "600"]),
            1,
            cx,
            |this: &mut Self, i, cx| {
                this.frames_index = i;
                cx.notify();
            },
        );
        let producers = Choice::new(
            "dense-count",
            "Independent producers",
            &look,
            labels(&["128", "512", "1,024"]),
            1,
            cx,
            |this: &mut Self, i, cx| {
                this.producers_index = i;
                cx.notify();
            },
        );
        let opacity = radix::Slider::new("quad-opacity")
            .look(&look)
            .range(0.05..1.)
            .step(0.05)
            .value(0.65)
            .on_value_changed(cx, |this: &mut Self, _, v, cx| {
                this.opacity_value = v;
                cx.notify();
            })
            .spawn(cx);
        let hue = radix::Slider::new("field-hue")
            .look(&look)
            .range(0.0..359.0)
            .step(1.)
            .value(210.)
            .on_value_changed(cx, |this: &mut Self, _, v, cx| {
                this.hue_value = v;
                cx.notify();
            })
            .spawn(cx);
        let buttons = ["Run benchmark", "Cancel", "Reveal results"]
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                radix::Button::new(format!("standalone-{i}"))
                    .look(&look)
                    .label(label)
                    .on_click(cx, move |this: &mut Self, cx| match i {
                        0 => this.start(cx),
                        1 => {
                            if this.busy {
                                this.cancel.store(true, Ordering::Relaxed);
                                this.status = "Cancelling…".into();
                            }
                            cx.notify();
                        }
                        _ => {
                            if let Some(output) = &this.output {
                                cx.reveal_path(output);
                            }
                        }
                    })
                    .spawn(cx)
            })
            .collect();
        Self {
            look,
            kind: Kind::Quad,
            buttons,
            count,
            pattern,
            method,
            samples,
            animation,
            backend,
            frames,
            producers,
            opacity,
            hue,
            count_index: 1,
            pattern_index: 0,
            method_index: 0,
            samples_index: 2,
            animation_index: 0,
            backend_index: 0,
            frames_index: 1,
            producers_index: 1,
            opacity_value: 0.65,
            hue_value: 210.,
            busy: false,
            cancel: Arc::new(AtomicBool::new(false)),
            status: "Choose settings, then run a bounded workload.".into(),
            output: None,
            report: None,
        }
    }
    pub fn activate(&mut self, page: &str, cx: &mut Context<Self>) {
        self.kind = match page {
            "Color Field Benchmark" => Kind::Field,
            "Dense Metal Producers" => Kind::Dense,
            _ => Kind::Quad,
        };
        cx.notify();
    }
    pub fn suspend(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
    fn args(&self) -> Vec<String> {
        if self.kind == Kind::Dense {
            return vec!["--producers".into(), [128, 512, 1024][self.producers_index].to_string()];
        }
        let mut args =
            vec!["--frames".into(), [5, 60, 600][self.frames_index].to_string(), "--warmup".into(), "10".into()];
        if self.backend_index == 1 {
            args.push("--cpu-only".into());
        }
        if self.kind == Kind::Quad {
            args.extend([
                "--count".into(),
                [100, 1000, 5000, 20000][self.count_index].to_string(),
                "--pattern".into(),
                if self.pattern_index == 0 {
                    "grid".into()
                } else {
                    "overlap".into()
                },
                "--opacity".into(),
                self.opacity_value.to_string(),
            ]);
        } else {
            if self.animation_index == 1 {
                args.push("--animate".into());
            }
            args.extend([
                "--method".into(),
                if self.method_index == 0 {
                    "layered".into()
                } else {
                    "sampled".into()
                },
                "--samples".into(),
                [16, 64, 128, 256, 512][self.samples_index].to_string(),
                "--hue".into(),
                self.hue_value.to_string(),
            ]);
        }
        args
    }
    fn start(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let output = match tempfile::Builder::new().prefix("xgpui-standalone-").tempdir() {
            Ok(dir) => dir.keep(),
            Err(e) => {
                self.status = e.to_string();
                cx.notify();
                return;
            }
        };
        self.busy = true;
        self.report = None;
        self.output = Some(output.clone());
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        let kind = self.kind;
        let args = self.args();
        self.status = format!("Running {}… settings are captured for this run.", kind.name());
        self.buttons[0].update(cx, |button, cx| button.set_enabled(false, cx));
        cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { run(kind, &args, &output, &cancel) }).await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(report) => {
                        this.status = format!("{} completed. Open results for timings and captures.", kind.name());
                        this.report = Some(report);
                    }
                    Err(e) => this.status = format!("{}: {e:#}", kind.name()),
                };
                this.buttons[0].update(cx, |button, cx| button.set_enabled(true, cx));
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}
impl Drop for Page {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let controls = div()
            .flex()
            .flex_wrap()
            .gap_4()
            .when(self.kind == Kind::Quad, |d| {
                d.child(self.count.clone()).child(self.pattern.clone()).child(
                    div().w(px(160.)).child(format!("Opacity {:.2}", self.opacity_value)).child(self.opacity.clone()),
                )
            })
            .when(self.kind == Kind::Field, |d| {
                d.child(self.method.clone())
                    .child(self.samples.clone())
                    .child(self.animation.clone())
                    .child(div().w(px(180.)).child(format!("Hue {:.0}°", self.hue_value)).child(self.hue.clone()))
            })
            .when(self.kind != Kind::Dense, |d| d.child(self.backend.clone()).child(self.frames.clone()))
            .when(self.kind == Kind::Dense, |d| d.child(self.producers.clone()));
        let chrome = self.look.chrome();
        let mut root = div()
            .bg(chrome.panel_background)
            .text_color(chrome.body_text)
            .id("standalone-benchmark-scroll")
            .size_full()
            .overflow_y_scroll()
            .p_5()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.kind.name())
            .child(controls)
            .child(div().flex().gap_2().children(self.buttons.iter().cloned()))
            .child(self.status.clone());
        if self.kind == Kind::Dense {
            root=root.child("macOS Metal · isolated control and dense/sparse/replay runs · 15-second timeout per process. A timeout is a failed regression. Preparation markers are retained in logs.");
        } else {
            root=root.child("Live preview uses the same geometry generator. Timings cover CPU work and optional offscreen rendering/readback, including GPU waits; they do not measure presented FPS or GPU-only time.");
            root = root.child(div().h(px(320.)).w_full().overflow_hidden().bg(rgb(0xf1f0e8)).child(
                if self.kind == Kind::Quad {
                    quad::preview(
                        [100, 1000, 5000, 20000][self.count_index],
                        self.pattern_index == 1,
                        self.opacity_value,
                    )
                    .into_any_element()
                } else {
                    field::preview(self.method_index == 1, [16, 64, 128, 256, 512][self.samples_index], self.hue_value)
                        .into_any_element()
                },
            ));
        }
        if let Some(report) = &self.report {
            root = root.child(div().text_size(px(12.)).child(serde_json::to_string_pretty(report).unwrap_or_default()));
        }
        if let Some(output) = &self.output {
            root = root.child(format!("Results: {}", output.display()));
        }
        root
    }
}
fn run(kind: Kind, args: &[String], output: &Path, cancel: &AtomicBool) -> Result<Value> {
    std::fs::create_dir_all(output)?;
    let result = (|| match kind {
        Kind::Quad => quad::run(args, output, cancel),
        Kind::Field => field::run(args, output, cancel),
        Kind::Dense => {
            ensure!(cfg!(target_os = "macos"), "Dense producers require native macOS Metal");
            let count = crate::benchmarks::argument(args, "--producers").unwrap_or("512");
            let mut cases = Vec::new();
            for control in [true, false] {
                let directory = output.join(if control { "control" } else { "dense" });
                cases.push(crate::benchmarks::run_self(
                    &[
                        "--dense-producer-worker".into(),
                        "--producers".into(),
                        count.into(),
                        "--control".into(),
                        control.to_string(),
                        "--output".into(),
                        directory.to_string_lossy().into_owned(),
                    ],
                    &directory,
                    cancel,
                )?);
            }
            Ok(
                json!({"cases":cases,"errors":[],"checks":["single producer control","dense/sparse/dense replacement","replay","exact linear P3 pixels"]}),
            )
        }
    })();
    let mut report = match result {
        Ok(report) => report,
        Err(error) => {
            let report = json!({"outcome":if cancel.load(Ordering::Relaxed){"cancelled"}else{"failed"},"errors":[format!("{error:#}")]});
            std::fs::write(output.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
            return Err(error);
        }
    };
    report["outcome"] = json!("completed");
    report["errors"] = json!([]);
    report["workload"] = json!(kind.slug());
    std::fs::write(output.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(report)
}
pub fn run_cli(args: &[String]) -> Result<()> {
    let output = PathBuf::from(
        crate::benchmarks::argument(args, "--output").ok_or_else(|| anyhow::anyhow!("--output required"))?,
    );
    if args.iter().any(|a| a == "--dense-producer-worker") {
        let count = crate::benchmarks::argument(args, "--producers").unwrap_or("512").parse::<u32>()?;
        ensure!((2..=4096).contains(&count), "producers must be 2..=4096");
        let control = crate::benchmarks::argument(args, "--control") == Some("true");
        dense::run(control, count)?;
        std::fs::create_dir_all(&output)?;
        std::fs::write(
            output.join("report.json"),
            serde_json::to_vec_pretty(
                &json!({"control":control,"producers":if control{1}else{count},"frames":3,"cycles":3,"errors":[]}),
            )?,
        )?;
        return Ok(());
    }
    let kind = match crate::benchmarks::argument(args, "--standalone-benchmark") {
        Some("quad") => Kind::Quad,
        Some("field") => Kind::Field,
        Some("dense") => Kind::Dense,
        _ => anyhow::bail!("expected quad, field, or dense"),
    };
    let mut forwarded = Vec::new();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--standalone-benchmark" || args[i] == "--output" {
            i += 2;
        } else {
            forwarded.push(args[i].clone());
            i += 1;
        }
    }
    run(kind, &forwarded, &output, &AtomicBool::new(false))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn cpu_workloads_keep_measurement_counts_and_cancel_without_success() {
        for kind in [Kind::Quad, Kind::Field] {
            let dir = tempfile::tempdir().unwrap();
            let args =
                ["--cpu-only", "--frames", "2", "--warmup", "1", "--width", "64", "--height", "64"].map(str::to_owned);
            let report = run(kind, &args, dir.path(), &AtomicBool::new(false)).unwrap();
            assert_eq!(report["measured_frames"], 2);
            assert_eq!(report["warmup_frames"], 1);
            assert_eq!(report["outcome"], "completed");
            assert!(!dir.path().join("capture.png").exists());
            assert!(run(kind, &args, dir.path(), &AtomicBool::new(true)).is_err());
            let report: Value =
                serde_json::from_slice(&std::fs::read(dir.path().join("report.json")).unwrap()).unwrap();
            assert_eq!(report["outcome"], "cancelled");
        }
    }
    #[cfg(feature = "test-support")]
    #[::core::prelude::v1::test]
    fn luma_selectors_preserve_settings_across_benchmark_pages() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| Page::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        let count = cx.update(|_, cx| page.read(cx).count.clone());
        cx.update(|_, cx| count.select(2, cx));
        page.update(cx, |page, cx| {
            assert_eq!(page.count_index, 2);
            assert!(page.args().contains(&"5000".into()));
            page.activate("Color Field Benchmark", cx);
        });
        let method = cx.update(|_, cx| page.read(cx).method.clone());
        cx.update(|_, cx| method.select(1, cx));
        page.update(cx, |page, cx| {
            assert!(page.args().contains(&"sampled".into()));
            page.activate("Quad Benchmark", cx);
            assert_eq!(page.count_index, 2);
            assert!(!page.busy);
        });
    }
}

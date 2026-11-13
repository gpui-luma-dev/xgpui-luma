//! Embedded Parley, Shader and SVG workloads. Isolated jobs launch only this executable.
pub(crate) mod memory;
pub(crate) mod progress;
pub(crate) mod svg_table;
mod svg_badge;
mod svg_chart;
mod svg_model;
mod svg_screen;
use anyhow::{Context as _, Result, ensure};
use gpui::{prelude::*, *};
use xgpui_luma::{controls::button::Button, prelude::HasPresenter};
use xgpui_luma_look_radix::{self as radix, Look};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

pub(crate) fn argument<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter().position(|arg| arg == flag).and_then(|i| args.get(i + 1)).map(String::as_str)
}
pub(crate) fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Relaxed), "benchmark cancelled");
    Ok(())
}
fn profile() -> &'static str {
    if cfg!(debug_assertions) { "debug" } else { "release" }
}

/// Fresh backend/corpus isolation without compilation, source-tree paths, or shell commands.
pub(crate) fn run_self(args: &[String], output: &Path, cancel: &AtomicBool) -> Result<Value> {
    check_cancel(cancel)?;
    std::fs::create_dir_all(output)?;
    let stdout = std::fs::File::create(output.join("stdout.log"))?;
    let stderr = std::fs::File::create(output.join("stderr.log"))?;
    let mut command = Command::new(std::env::current_exe()?);
    command.args(args).stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn().context("starting this app's isolated benchmark")?;
    let status = wait_child(&mut child, args, cancel)?;
    ensure!(
        status.success(),
        "isolated benchmark exited {status}; inspect {}",
        output.join("stderr.log").display()
    );
    let report: Value = serde_json::from_slice(
        &std::fs::read(output.join("report.json")).context("benchmark did not produce report.json")?,
    )?;
    validate_report(&report)?;
    Ok(report)
}
fn validate_report(report: &Value) -> Result<()> {
    ensure!(
        !report.get("errors").and_then(Value::as_array).is_some_and(|e| !e.is_empty()),
        "isolated benchmark reported errors"
    );
    Ok(())
}
fn wait_child(
    child: &mut std::process::Child,
    args: &[String],
    cancel: &AtomicBool,
) -> Result<std::process::ExitStatus> {
    let started = Instant::now();
    let timeout = if args.iter().any(|arg| arg == "--dense-producer-worker") {
        Duration::from_secs(15)
    } else if args.iter().any(|arg| arg == "--lifecycle-worker") {
        Duration::from_secs(360)
    } else if args.iter().any(|arg| arg == "--svg-case") {
        Duration::from_secs(600)
    } else {
        Duration::from_secs(120)
    };
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if cancel.load(Ordering::Relaxed) || started.elapsed() > timeout {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGTERM);
            }
            #[cfg(not(unix))]
            {
                let _ = child.kill();
            }
            let stop = Instant::now();
            while child.try_wait()?.is_none() {
                if stop.elapsed() > Duration::from_secs(2) {
                    #[cfg(unix)]
                    unsafe {
                        libc::kill(-(child.id() as i32), libc::SIGKILL);
                    }
                    let _ = child.kill();
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            let _ = child.wait();
            anyhow::bail!("isolated benchmark cancelled or exceeded {} seconds", timeout.as_secs());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suite {
    Parley,
    Shader,
    Svg,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Job {
    TextCpu,
    NativeScroll,
    ParleyScroll,
    Atlas,
    Paint,
    Mesh,
    Wgsl,
    Grid,
    Seascape,
    SvgBuiltin,
    SvgSelected,
    SvgCorpus,
}
impl Suite {
    fn jobs(self) -> Vec<Job> {
        match self {
            Self::Parley => vec![Job::TextCpu, Job::NativeScroll, Job::ParleyScroll, Job::Atlas],
            Self::Shader => vec![Job::Paint, Job::Mesh, Job::Wgsl, Job::Grid, Job::Seascape],
            Self::Svg => vec![Job::SvgBuiltin, Job::SvgSelected, Job::SvgCorpus],
        }
    }
}
impl Job {
    fn id(self) -> &'static str {
        match self {
            Self::TextCpu => "cpu",
            Self::NativeScroll => "native-scroll",
            Self::ParleyScroll => "parley-scroll",
            Self::Atlas => "atlas",
            Self::Paint => "paint",
            Self::Mesh => "mesh",
            Self::Wgsl => "wgsl",
            Self::Grid => "grid",
            Self::Seascape => "seascape",
            Self::SvgBuiltin => "builtin",
            Self::SvgSelected => "selected",
            Self::SvgCorpus => "corpus",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::TextCpu => "Native / Parley CPU comparison",
            Self::NativeScroll => "Native scroll · 600 frames",
            Self::ParleyScroll => "Parley scroll · 600 frames",
            Self::Atlas => "Parley glyph atlas capture",
            Self::Paint => "Paint coverage checks",
            Self::Mesh => "Mesh checks",
            Self::Wgsl => "WGSL checks",
            Self::Grid => "Shader Depot Grid · public API parity",
            Self::Seascape => "Shadertoy Seascape · looped / expanded WGSL",
            Self::SvgBuiltin => "Built-in SVG",
            Self::SvgSelected => "Selected SVG",
            Self::SvgCorpus => "Bundled SVG corpus",
        }
    }
}
fn text_job(backend: &str, mode: &str, output: &Path, cancel: &AtomicBool) -> Result<Value> {
    let args = vec![
        "--text-benchmark-backend".into(),
        backend.into(),
        "--text-benchmark-mode".into(),
        mode.into(),
        "--output".into(),
        output.display().to_string(),
    ];
    run_self(&args, output, cancel)
}
fn run_job(
    job: Job,
    output: &Path,
    selected: Option<&Path>,
    corpus: Option<&Path>,
    cancel: &AtomicBool,
) -> Result<Value> {
    std::fs::create_dir_all(output)?;
    check_cancel(cancel)?;
    match job {
        Job::TextCpu => {
            let native = text_job("native", "cpu", &output.join("native"), cancel)?;
            let parley = text_job("parley", "cpu", &output.join("parley"), cancel)?;
            ensure!(
                native["sample_labels"] == parley["sample_labels"],
                "text backends measured different sample corpora"
            );
            Ok(
                json!({"native": native, "parley": parley, "summary": "Both backends measured in fresh copies of this executable with the same embedded font/sample corpus.", "errors": []}),
            )
        }
        Job::NativeScroll => text_job("native", "scroll", output, cancel),
        Job::ParleyScroll => text_job("parley", "scroll", output, cancel),
        Job::Atlas => text_job("parley", "atlas", output, cancel),
        Job::Grid => crate::shader::benchmarks::grid_checks::run(output, cancel),
        Job::Seascape => crate::shader::benchmarks::seascape_checks::run(output, cancel),
        Job::Paint | Job::Mesh | Job::Wgsl => crate::shader::benchmarks::run(job.id(), output, cancel),
        Job::SvgBuiltin => crate::svg_lab::benchmarks::builtin(output, cancel),
        Job::SvgSelected => match selected {
            Some(path) => crate::svg_lab::benchmarks::selected(path, output, cancel),
            None => Ok(json!({"outcome":"unavailable","summary":"Choose an SVG file to run this workload."})),
        },
        // The same sweep the page runs, so a saved report.json replays with `--svg-results`.
        Job::SvgCorpus => crate::svg_lab::benchmarks::memory_corpus(
            corpus,
            &crate::svg_lab::benchmarks::SWEEP_ZOOMS,
            &crate::svg_lab::benchmarks::COMPARISON_BUDGETS,
            cancel,
            &progress::Progress::default(),
        ),
    }
}
fn decorate(job: Job, result: Result<Value>) -> Value {
    let mut report = result.unwrap_or_else(|error| json!({"error":format!("{error:#}")}));
    report["job"] = json!(job.id());
    report["label"] = json!(job.label());
    report["profile"] = json!(profile());
    report["os"] = json!(std::env::consts::OS);
    report["arch"] = json!(std::env::consts::ARCH);
    let failed = report.get("error").is_some()
        || report.get("errors").and_then(Value::as_array).is_some_and(|errors| !errors.is_empty());
    if failed {
        report["outcome"] = json!("failed");
    } else if report.get("outcome").is_none() {
        report["outcome"] = json!("completed");
    }
    report
}
fn record(job: Job, result: Result<Value>, output: &Path) -> Result<Value> {
    let mut report = decorate(job, result);
    let mut previews = std::fs::read_dir(output)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .collect::<Vec<_>>();
    previews.sort();
    report["previews"] = json!(previews);
    std::fs::write(output.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(report)
}
fn summary(output: &Path, results: &[Value], cancelled: bool) -> Result<()> {
    std::fs::create_dir_all(output)?;
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&json!({"cases":results,"cancelled":cancelled,"profile":profile()}))?,
    )?;
    Ok(())
}
pub struct Page {
    suite: Suite,
    look: Arc<Look>,
    buttons: Vec<Entity<Button>>,
    cancel_button: Entity<Button>,
    reveal: Entity<Button>,
    selected: Option<PathBuf>,
    corpus: Option<PathBuf>,
    cancel: Arc<AtomicBool>,
    busy: bool,
    svg_zooms: Vec<f32>,
    svg_budgets: Vec<usize>,
    svg_cases: Vec<Value>,
    svg_zoom_choice: crate::lab_controls::Segmented,
    /// Facets (one raster chart per artwork) or one merged chart.
    svg_raster_view: crate::lab_controls::Segmented,
    svg_raster_merged: bool,
    svg_budget_choice: crate::lab_controls::Segmented,
    svg_tabs: Entity<xgpui_luma::controls::tabs::Tabs>,
    svg_tab: svg_screen::Tab,
    svg_progress_bar: xgpui_luma::controls::progress::Progress,
    svg_started: Option<Instant>,
    status: String,
    output: Option<PathBuf>,
    results: Vec<Value>,
    svg_table: xgpui_luma::controls::table::Table<svg_table::Row>,
    svg_thumbnails: svg_table::Thumbnails,
    thumbnail_request: u64,
    quit_on_finish: bool,
}
impl Drop for Page {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl Page {
    pub fn new(suite: Suite, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let actions = if suite == Suite::Svg {
            vec![(None, "Run benchmark")]
        } else {
            std::iter::once((None, "Run all"))
                .chain(suite.jobs().into_iter().map(|job| (Some(job), job.label())))
                .collect()
        };
        let buttons = actions
            .into_iter()
            .enumerate()
            .map(|(index, (job, label))| {
                radix::Button::new(format!("bench-{suite:?}-{index}"))
                    .look(&look)
                    .outline()
                    .label(label)
                    .on_click(cx, move |this: &mut Self, cx| this.start(job, None, cx))
                    .spawn(cx)
            })
            .collect();
        let cancel_button = radix::Button::new("bench-cancel")
            .look(&look)
            .outline()
            .label("Stop")
            .enabled(false)
            .on_click(cx, |this: &mut Self, cx| this.stop(cx))
            .spawn(cx);
        let reveal = radix::Button::new("bench-reveal")
            .look(&look)
            .outline()
            .label("Reveal results")
            .on_click(cx, |this: &mut Self, cx| {
                if let Some(path) = &this.output {
                    cx.reveal_path(path);
                }
            })
            .spawn(cx);
        let svg_raster_view = crate::lab_controls::Segmented::new(
            "svg-raster-view",
            "View",
            &look,
            &["Facets", "Merged"],
            0,
            cx,
            |this: &mut Self, index, cx| {
                this.svg_raster_merged = index == 1;
                cx.notify();
            },
        );
        let svg_zoom_choice = crate::lab_controls::Segmented::new(
            "svg-comparison-zoom",
            "Zoom",
            &look,
            &svg_screen::ZOOM_LABELS,
            0,
            cx,
            |this: &mut Self, index, cx| {
                this.svg_zooms = svg_screen::zooms(index);
                this.reset_comparison(cx);
            },
        );
        let svg_budget_choice = crate::lab_controls::Segmented::new(
            "svg-comparison-budget",
            "Budget",
            &look,
            &svg_screen::BUDGET_LABELS,
            2,
            cx,
            |this: &mut Self, index, cx| {
                this.svg_budgets = svg_screen::budgets(index);
                this.reset_comparison(cx);
            },
        );
        let svg_tabs = radix::Tabs::new("svg-benchmark-tabs")
            .look(&look)
            .line()
            .items(
                svg_screen::Tab::ALL
                    .iter()
                    .map(|(_, id, label)| xgpui_luma::controls::tabs::TabsItem::new(*id).label(*label)),
            )
            .active(svg_screen::Tab::ALL[0].1)
            .on_value_changed(cx, |this: &mut Self, id, _, cx| {
                this.svg_tab = svg_screen::Tab::from_id(id);
                cx.notify();
            })
            .spawn(cx);
        let svg_progress_bar = radix::Progress::new("svg-benchmark-progress").look(&look).value(0.).spawn(cx);
        let (svg_zooms, svg_budgets) = (svg_screen::zooms(0), svg_screen::budgets(2));
        let svg_cases = if suite == Suite::Svg {
            svg_table::plan(&[Job::SvgCorpus], &svg_zooms, &svg_budgets)
        } else {
            vec![]
        };
        let svg_table = svg_table::build(&look, cx);
        if suite == Suite::Svg {
            svg_table.update(cx, |table, cx| {
                let _ = table.set_items(svg_table::rows(&svg_cases), cx);
            });
        }
        let mut page = Self {
            svg_thumbnails: Default::default(),
            thumbnail_request: 0,
            svg_table,
            suite,
            look,
            buttons,
            cancel_button,
            reveal,
            selected: None,
            corpus: None,
            cancel: Arc::new(AtomicBool::new(false)),
            busy: false,
            svg_zooms,
            svg_budgets,
            svg_zoom_choice,
            svg_budget_choice,
            svg_tabs,
            svg_tab: svg_screen::Tab::Graph,
            svg_raster_view,
            svg_raster_merged: false,
            svg_progress_bar,
            svg_started: None,
            svg_cases,
            status: if suite == Suite::Svg {
                ""
            } else {
                "Choose a workload. Reports and captures use the embedded application code."
            }
            .into(),
            output: None,
            results: Vec::new(),
            quit_on_finish: false,
        };
        if suite == Suite::Svg {
            page.reset_comparison(cx);
            page.load_thumbnails(cx);
            let args: Vec<String> = std::env::args().collect();
            if let Some(source) = argument(&args, "--svg-results") {
                page.replay(source, cx);
            }
            if argument(&args, "--svg-view") == Some("merged") {
                page.svg_raster_merged = true;
                page.svg_raster_view.group.update(cx, |group, cx| group.set_selected("1", cx));
            }
            if argument(&args, "--svg-tab") == Some("table") {
                page.svg_tab = svg_screen::Tab::Table;
                page.svg_tabs.update(cx, |tabs, cx| tabs.set_active("table", cx));
            }
        }
        page
    }
    fn comparison_jobs(&self) -> Vec<Job> {
        let mut jobs = vec![Job::SvgCorpus];
        if self.selected.is_some() {
            jobs.push(Job::SvgSelected);
        }
        jobs
    }
    fn comparison_plan(&self, jobs: &[Job]) -> Vec<Value> {
        let mut plan = svg_table::plan(jobs, &self.svg_zooms, &self.svg_budgets);
        if let Some(path) = self.selected.as_deref()
            && let Some(name) = path.file_name()
        {
            let bytes = std::fs::metadata(path).ok().map(|meta| meta.len());
            for row in &mut plan {
                if row["id"] == "Selected SVG" {
                    row["display_name"] = json!(name.to_string_lossy());
                    row["coverage"] = json!("added artwork");
                    if let Some(bytes) = bytes {
                        row["input_bytes"] = json!(bytes);
                    }
                }
            }
        }
        plan
    }
    /// Show saved results instead of the empty plan: `sample` for made-up data, or a run's `report.json`.
    fn replay(&mut self, source: &str, cx: &mut Context<Self>) {
        let loaded = if source == "sample" {
            Ok(svg_model::sample_cases())
        } else {
            std::fs::read(source).map_err(anyhow::Error::from).and_then(|bytes| {
                let report: Value = serde_json::from_slice(&bytes)?;
                let reports = report["cases"].as_array().or(report.as_array()).context("no cases in report")?;
                Ok(svg_table::flatten(reports))
            })
        };
        match loaded {
            Ok(cases) => {
                self.status = if source == "sample" {
                    format!("Showing {} made-up sample cases, not measurements.", cases.len())
                } else {
                    format!("Showing {} saved cases from {source}.", cases.len())
                };
                self.svg_cases = cases;
                let rows = svg_table::with_thumbnails(svg_table::rows(&self.svg_cases), &self.svg_thumbnails);
                self.svg_table.update(cx, |table, cx| {
                    let _ = table.set_items(rows, cx);
                });
            }
            Err(error) => self.status = format!("Cannot load {source}: {error:#}"),
        }
        cx.notify();
    }
    fn reset_comparison(&mut self, cx: &mut Context<Self>) {
        if self.suite != Suite::Svg || self.busy {
            return;
        }
        self.results.clear();
        self.svg_cases = self.comparison_plan(&self.comparison_jobs());
        let rows = svg_table::with_thumbnails(svg_table::rows(&self.svg_cases), &self.svg_thumbnails);
        self.svg_table.update(cx, |table, cx| {
            let _ = table.set_items(rows, cx);
        });
        self.status.clear();
        cx.notify();
    }
    fn load_thumbnails(&mut self, cx: &mut Context<Self>) {
        self.svg_thumbnails.clear();
        let rows = svg_table::with_thumbnails(self.svg_table.read(cx).items().to_vec(), &self.svg_thumbnails);
        self.svg_table.update(cx, |table, cx| {
            let _ = table.set_items(rows, cx);
        });
        self.thumbnail_request += 1;
        let request = self.thumbnail_request;
        let selected = self.selected.clone();
        let corpus = self.corpus.clone();
        cx.spawn(async move |this, cx| {
            let images = cx
                .background_spawn(async move {
                    crate::svg_lab::benchmarks::thumbnails(selected.as_deref(), corpus.as_deref())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.thumbnail_request != request {
                    return;
                }
                this.svg_thumbnails = images;
                let rows = svg_table::with_thumbnails(this.svg_table.read(cx).items().to_vec(), &this.svg_thumbnails);
                this.svg_table.update(cx, |table, cx| {
                    let _ = table.set_items(rows, cx);
                });
                cx.notify();
            });
        })
        .detach();
    }
    fn stop(&mut self, cx: &mut Context<Self>) {
        if !self.busy || self.cancel.swap(true, Ordering::Relaxed) {
            return;
        }
        self.status = "Stopping active workers…".into();
        self.cancel_button.update(cx, |button, cx| {
            button.set_label("Stopping…", cx);
            button.set_enabled(false, cx);
        });
        cx.notify();
    }
    fn start(&mut self, job: Option<Job>, output: Option<PathBuf>, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if job == Some(Job::SvgSelected) && self.selected.is_none() {
            self.status = "Choose an SVG file first.".into();
            cx.notify();
            return;
        }
        let in_memory = self.suite == Suite::Svg && output.is_none();
        let output = if in_memory {
            None
        } else {
            Some(
                match output.map_or_else(
                    || tempfile::Builder::new().prefix("xgpui-benchmarks-").tempdir().map(|dir| dir.keep()),
                    Ok,
                ) {
                    Ok(output) => output,
                    Err(error) => {
                        self.status = format!("Cannot create results: {error}");
                        cx.notify();
                        return;
                    }
                },
            )
        };
        for choice in [&self.svg_zoom_choice, &self.svg_budget_choice] {
            choice.set_enabled(false, cx);
        }
        self.svg_started = Some(Instant::now());
        self.busy = true;
        self.cancel_button.update(cx, |button, cx| {
            button.set_label("Stop", cx);
            button.set_enabled(true, cx);
        });
        self.status = "Preparing benchmark…".into();
        self.results.clear();
        self.svg_table.update(cx, |table, cx| {
            let _ = table.set_items(Vec::new(), cx);
        });
        self.output = output.clone();
        self.cancel = Arc::new(AtomicBool::new(false));
        for button in &self.buttons {
            button.update(cx, |button, cx| button.set_enabled(false, cx));
        }
        let jobs = job.map_or_else(
            || {
                if in_memory {
                    self.comparison_jobs()
                } else {
                    self.suite.jobs()
                }
            },
            |job| vec![job],
        );
        let cancel = self.cancel.clone();
        let selected = self.selected.clone();
        let corpus = self.corpus.clone();
        let (zooms, budgets) = (self.svg_zooms.clone(), self.svg_budgets.clone());
        let progress = progress::Progress::default();
        if in_memory {
            let plan = self.comparison_plan(&jobs);
            self.svg_cases = plan.clone();
            self.svg_table.update(cx, |table, cx| {
                let _ = table.set_items(svg_table::with_thumbnails(svg_table::rows(&plan), &self.svg_thumbnails), cx);
            });
            progress.plan(plan);
        }
        if in_memory {
            let progress = progress.clone();
            let token = cancel.clone();
            cx.spawn(async move |this, cx| {
                let mut last_rows = Vec::new();
                loop {
                    cx.background_executor().timer(Duration::from_millis(200)).await;
                    let (_, partial) = progress.snapshot();
                    let status = progress.headline();
                    let keep_running = this
                        .update(cx, |this, cx| {
                            if !this.busy || !Arc::ptr_eq(&this.cancel, &token) {
                                return false;
                            }
                            this.status = if token.load(Ordering::Relaxed) {
                                format!("Cancelling · {status}")
                            } else {
                                status
                            };
                            if partial != last_rows {
                                this.svg_cases = partial.clone();
                                let rows = svg_table::with_thumbnails(svg_table::rows(&partial), &this.svg_thumbnails);
                                this.svg_table.update(cx, |table, cx| {
                                    let _ = table.set_items(rows, cx);
                                });
                                last_rows = partial.clone();
                            }
                            cx.notify();
                            true
                        })
                        .unwrap_or(false);
                    if !keep_running {
                        break;
                    }
                }
            })
            .detach();
        }
        cx.spawn(async move |this, cx| {
            for job in jobs {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                if this
                    .update(cx, |this, cx| {
                        this.status = format!("Running {}…", job.label());
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }
                progress.case(job.label());
                let directory = output.as_ref().map(|output| output.join(job.id()));
                let token = cancel.clone();
                let selected = selected.clone();
                let corpus = corpus.clone();
                let worker_progress = progress.clone();
                let (zooms, budgets) = (zooms.clone(), budgets.clone());
                let result = cx
                    .background_spawn(async move {
                        if let Some(directory) = directory {
                            let result = run_job(job, &directory, selected.as_deref(), corpus.as_deref(), &token);
                            std::fs::create_dir_all(&directory)?;
                            record(job, result, &directory)
                        } else {
                            let mut report = decorate(
                                job,
                                run_svg_memory(
                                    job,
                                    selected.as_deref(),
                                    corpus.as_deref(),
                                    &zooms,
                                    &budgets,
                                    &token,
                                    &worker_progress,
                                ),
                            );
                            if token.load(Ordering::Relaxed) {
                                report["outcome"] = json!("cancelled");
                            }
                            Ok(report)
                        }
                    })
                    .await;
                if this
                    .update(cx, |this, cx| {
                        this.results.push(result.unwrap_or_else(
                            |error| json!({"label":job.label(),"outcome":"failed","error":format!("{error:#}")}),
                        ));
                        let rows = if in_memory {
                            this.svg_cases = progress.snapshot().1;
                            svg_table::rows(&this.svg_cases)
                        } else {
                            svg_table::rows(&this.results)
                        };
                        let rows = svg_table::with_thumbnails(rows, &this.svg_thumbnails);
                        this.svg_table.update(cx, |table, cx| {
                            let _ = table.set_items(rows, cx);
                        });
                        if let Some(output) = &output
                            && let Err(error) = summary(output, &this.results, cancel.load(Ordering::Relaxed))
                        {
                            this.results
                                .push(json!({"outcome":"failed","error":format!("Cannot save results: {error:#}")}));
                            cancel.store(true, Ordering::Relaxed);
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }
            }
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                this.cancel_button.update(cx, |button, cx| {
                    button.set_label("Stop", cx);
                    button.set_enabled(false, cx);
                });
                if in_memory {
                    progress.settle(None, cancel.load(Ordering::Relaxed));
                    this.svg_cases = progress.snapshot().1;
                    let rows = svg_table::with_thumbnails(svg_table::rows(&this.svg_cases), &this.svg_thumbnails);
                    this.svg_table.update(cx, |table, cx| {
                        let _ = table.set_items(rows, cx);
                    });
                }
                for choice in [&this.svg_zoom_choice, &this.svg_budget_choice] {
                    choice.set_enabled(true, cx);
                }
                if let Some(output) = &output
                    && let Err(error) = summary(output, &this.results, cancel.load(Ordering::Relaxed))
                {
                    this.results.push(json!({"outcome":"failed","error":format!("Cannot save summary: {error:#}")}));
                }
                let failed = this.results.iter().filter(|result| result["outcome"] == "failed").count();
                let unavailable = this.results.iter().filter(|result| result["outcome"] == "unavailable").count();
                let verb = if cancel.load(Ordering::Relaxed) {
                    "Cancelled"
                } else {
                    "Finished"
                };
                this.status = if in_memory {
                    let cases = svg_model::cases(&this.svg_cases);
                    let measured = cases.iter().filter(|case| case.measure.is_some()).count();
                    let failed = cases.iter().filter(|case| case.outcome == svg_model::Outcome::Failed).count();
                    format!(
                        "{verb} in {:.1}s: {measured} of {} cases measured, {failed} failed.",
                        progress.elapsed(),
                        cases.len()
                    )
                } else {
                    format!(
                        "{verb} in {:.1}s: {} workloads, {failed} failures, {unavailable} unavailable.",
                        progress.elapsed(),
                        this.results.len()
                    )
                };
                for button in &this.buttons {
                    button.update(cx, |button, cx| button.set_enabled(true, cx));
                }
                cx.notify();
                if this.quit_on_finish {
                    if failed > 0 || cancel.load(Ordering::Relaxed) {
                        eprintln!("{}", this.status);
                        std::process::exit(1);
                    }
                    cx.quit();
                }
            });
        })
        .detach();
        cx.notify();
    }
}
impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::theme::ColorVizLookExt;
        if self.suite == Suite::Svg {
            return self.render_svg(cx);
        }
        let body=div().id("embedded-benchmarks").flex_1().min_h_0().overflow_y_scroll().p_4().flex().flex_col().gap_4().text_color(self.look.chrome().body_text)
            .child(div().flex().flex_wrap().gap_2().children(self.buttons.clone()))
            .child(match self.suite{Suite::Parley=>"Both text backends use the same embedded samples and fonts. Scroll cases run 600 frames in a temporary window. Backend isolation uses this app’s executable.",Suite::Shader=>"Native pixel checks cover geometry, gradients, alpha, clipping, replay, resize, source reuse and portable WGSL. Timings include readback, not GPU-only time.",Suite::Svg=>""})
            .child(self.status.clone()).when_some(self.output.as_ref(),|root,output|root.child(format!("Results: {}",output.display())).child(self.reveal.clone()))
            .children(self.results.iter().map(|result|{
                let mut row=div().flex().flex_col().gap_2().child(format!("{} · {}",result["label"].as_str().unwrap_or("Workload"),result["outcome"].as_str().unwrap_or("unknown")));
                for key in ["error","summary","measurement","stats"]{if let Some(text)=result[key].as_str(){row=row.child(text.to_owned());}}
                if let Some(errors)=result["errors"].as_array(){for error in errors{row=row.child(error.to_string());}}
                if let Some(ms)=result["raster"]["median_ms"].as_f64(){row=row.child(format!("Raster median: {ms:.3} ms · {}×{} pixels",result["pixel_width"],result["pixel_height"]));}
                if let Some(previews)=result["previews"].as_array(){for path in previews.iter().filter_map(Value::as_str){row=row.child(img(PathBuf::from(path)).max_w(px(760.)));}}row
            }));
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("benchmark-run-controls")
                    .debug_selector(|| "benchmark-run-controls".into())
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .p_4()
                    .child(crate::screen::text(
                        &self.look,
                        crate::screen::TextRole::Name,
                        format!("{:?} benchmarks · {} build", self.suite, profile()),
                    ))
                    .child(self.cancel_button.clone()),
            )
            .child(body)
            .into_any_element()
    }
}
pub fn run_cli(args: &[String], suite: Suite, flag: &str) -> Result<()> {
    let name = argument(args, flag).context("benchmark case required")?;
    let job = if name == "all" {
        None
    } else {
        Some(suite.jobs().into_iter().find(|job| job.id() == name).context("unknown benchmark case")?)
    };
    let output = PathBuf::from(argument(args, "--output").context("output directory required")?);
    std::fs::create_dir_all(&output)?;
    let output = output.canonicalize()?;
    let selected = argument(args, "--input").map(PathBuf::from);
    let corpus = argument(args, "--corpus").map(PathBuf::from);
    ensure!(
        job != Some(Job::SvgSelected) || selected.is_some(),
        "selected SVG benchmark requires --input <file>"
    );
    gpui_platform::application().with_assets(crate::assets::Assets).run(move |cx| {
        if let Err(error) = xgpui_luma::init(cx) {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1100.), px(900.)), cx))),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Page::new(suite, crate::theme::default_look(), cx)),
        );
        match opened {
            Ok(handle) => {
                let _ = handle.update(cx, |page, _, cx| {
                    page.quit_on_finish = true;
                    page.selected = selected;
                    page.corpus = corpus;
                    page.start(job, Some(output), cx);
                });
            }
            Err(error) => {
                eprintln!("{error:#}");
                std::process::exit(1);
            }
        }
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
    });
    Ok(())
}
fn run_svg_memory(
    job: Job,
    selected: Option<&Path>,
    corpus: Option<&Path>,
    zooms: &[f32],
    budgets: &[usize],
    cancel: &AtomicBool,
    progress: &progress::Progress,
) -> Result<Value> {
    if job == Job::SvgCorpus {
        let result = crate::svg_lab::benchmarks::memory_corpus(corpus, zooms, budgets, cancel, progress);
        if result.is_err() {
            progress.settle(Some(job.id()), cancel.load(Ordering::Relaxed));
        }
        return result;
    }
    let path = match job {
        Job::SvgBuiltin => None,
        Job::SvgSelected => Some(selected.context("Choose an SVG file first")?),
        _ => anyhow::bail!("expected SVG workload"),
    };
    let mut cases = Vec::new();
    let total = zooms.len() * budgets.len();
    let tasks = zooms.iter().flat_map(|zoom| budgets.iter().map(move |budget| (*zoom, *budget)));
    for (index, (zoom, budget)) in tasks.enumerate() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        progress.case(format!(
            "Case {} of {total} · {} · {zoom}× · {} MiB",
            index + 1,
            job.label(),
            budget / xgpui_svg::MIB
        ));
        progress.running_budget(job.label(), zoom as f64, budget);
        let result = crate::svg_lab::benchmarks::isolated_selected(path, budget, zoom, cancel, progress);
        let mut report = result.unwrap_or_else(|e| json!({"error":format!("{e:#}")}));
        report["id"] = json!(job.label());
        report["job"] = json!(job.id());
        report["zoom"] = json!(zoom);
        report["budget_bytes"] = json!(budget);
        if cancel.load(Ordering::Relaxed) {
            report["outcome"] = json!("cancelled");
        }
        progress.completed(report.clone());
        cases.push(report);
    }
    let failed = cases.iter().filter(|r| r.get("error").is_some() && r["outcome"] != "cancelled").count();
    Ok(
        json!({"cases":cases,"errors":if failed==0 {vec![]}else{vec![format!("{failed} cases failed")]},"measurement":"SVG budget and zoom comparison at 2× density, serial isolated CPU workers. Five parse samples, then five raster repeats of a retained tree. No GPU timing."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn cancellation_prevents_self_launch_and_records_empty_runs() {
        let dir = tempfile::tempdir().unwrap();
        assert!(run_self(&[], dir.path(), &AtomicBool::new(true)).is_err());
        summary(dir.path(), &[], true).unwrap();
        let report: Value = serde_json::from_slice(&std::fs::read(dir.path().join("report.json")).unwrap()).unwrap();
        assert_eq!(report["cancelled"], true);
    }
    #[cfg(feature = "test-support")]
    #[::core::prelude::v1::test]
    fn the_view_toggle_switches_between_facets_and_one_merged_chart() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| Page::new(Suite::Svg, crate::theme::default_look(), cx));
        page.update(cx, |page, cx| page.replay("sample", cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("svg-raster-facet-3").is_some(), "four facets by default");
        assert!(cx.debug_bounds("svg-raster-merged").is_none());
        let view = cx.update(|_, cx| page.read(cx).svg_raster_view.clone());
        cx.update(|_, cx| view.select(1, cx));
        cx.run_until_parked();
        assert!(cx.update(|_, cx| page.read(cx).svg_raster_merged));
        assert!(cx.debug_bounds("svg-raster-merged").is_some(), "one merged chart");
        assert!(cx.debug_bounds("svg-raster-facet-0").is_none());
    }
    #[cfg(feature = "test-support")]
    #[::core::prelude::v1::test]
    fn stop_button_cancels_the_run_and_retains_planned_rows_without_files() {
        use xgpui_luma::controls::button::ButtonEvent;
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| Page::new(Suite::Svg, crate::theme::default_look(), cx));
        cx.update(|_, cx| assert_eq!(page.read(cx).svg_table.read(cx).items().len(), 40));
        page.update(cx, |page, cx| {
            page.start(Some(Job::SvgBuiltin), None, cx);
            assert_eq!(page.svg_table.read(cx).items().len(), 10);
        });
        let stop = cx.update(|_, cx| page.read(cx).cancel_button.clone());
        stop.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert!(page.cancel.load(Ordering::Relaxed));
            assert_eq!(page.status, "Stopping active workers…");
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert!(!page.busy);
            assert!(page.output.is_none());
            assert!(page.status.starts_with("Cancelled"), "{}", page.status);
            assert_eq!(page.svg_table.read(cx).items().len(), 10);
        });
        assert!(
            cx.debug_bounds("svg-benchmarks-controls").is_some(),
            "Stop remains in the control row after stopping"
        );
    }
    #[cfg(unix)]
    #[::core::prelude::v1::test]
    fn stop_terminates_and_reaps_the_active_worker_process() {
        use std::os::unix::process::CommandExt;
        let mut child = Command::new("/bin/sleep")
            .arg("60")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .unwrap();
        let start = Instant::now();
        assert!(wait_child(&mut child, &["--svg-case".into()], &AtomicBool::new(true)).is_err());
        assert!(child.try_wait().unwrap().is_some(), "worker has been reaped");
        assert!(start.elapsed() < Duration::from_secs(4));
    }
    #[cfg(feature = "test-support")]
    #[::core::prelude::v1::test]
    fn added_svg_budget_and_zoom_shape_the_plan_and_the_page_fits() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| Page::new(Suite::Svg, crate::theme::default_look(), cx));
        page.update(cx, |page, cx| {
            assert_eq!(page.svg_cases.len(), 40, "4 artworks × 5 zooms × 2 budgets");
            page.selected = Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/samples/svg/camera.svg"));
            page.reset_comparison(cx);
            page.load_thumbnails(cx);
            assert_eq!(page.svg_cases.len(), 50);
            let added: Vec<_> = page.svg_cases.iter().filter(|r| r["id"] == "Selected SVG").collect();
            assert_eq!(added.len(), 10);
            assert_eq!(added[0]["budget_bytes"], 16 * xgpui_svg::MIB);
            assert_eq!(added[1]["budget_bytes"], 32 * xgpui_svg::MIB);
            assert!(added[0]["input_bytes"].as_u64().is_some_and(|bytes| bytes > 0));
        });
        cx.run_until_parked();
        let (zoom, budget) = cx.update(|_, cx| {
            let page = page.read(cx);
            (page.svg_zoom_choice.clone(), page.svg_budget_choice.clone())
        });
        cx.update(|_, cx| zoom.select(3, cx));
        cx.update(|_, cx| budget.select(0, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let p = page.read(cx);
            assert_eq!(p.svg_cases.len(), 5);
            assert!(p.svg_cases.iter().all(|r| r["zoom"].as_f64() == Some(16.)));
            assert!(p.svg_cases.iter().all(|r| r["budget_bytes"] == 16 * xgpui_svg::MIB));
            assert!(p.svg_thumbnails.contains_key("Selected SVG"));
        });
        for mode in [xgpui_luma::theme::ThemeMode::Dark, xgpui_luma::theme::ThemeMode::Light] {
            page.update(cx, |page, cx| {
                page.look.set_mode(mode);
                cx.notify();
            });
            for tab in [svg_screen::Tab::Graph, svg_screen::Tab::Table] {
                page.update(cx, |page, cx| {
                    page.svg_tab = tab;
                    cx.notify();
                });
                for width in [1280., 720.] {
                    cx.simulate_resize(size(px(width), px(3200.)));
                    cx.run_until_parked();
                    let mut selectors = vec!["svg-benchmarks-controls", "svg-artwork-card-0", "svg-benchmark-results"];
                    selectors.extend(if tab == svg_screen::Tab::Graph {
                        &["svg-raster-chart", "svg-output-chart", "svg-cost-chart", "svg-graph-row-1"][..]
                    } else {
                        &["svg-comparison-table-scroll"][..]
                    });
                    for selector in selectors {
                        let bounds = cx.debug_bounds(selector).unwrap_or_else(|| panic!("{selector} missing"));
                        assert!(bounds.right() <= px(width), "{selector} overflows {width}");
                    }
                }
            }
        }
    }
    #[::core::prelude::v1::test]
    fn report_errors_are_failures_not_success() {
        let dir = tempfile::tempdir().unwrap();
        let result = record(Job::SvgCorpus, Ok(json!({"errors":["hash mismatch"]})), dir.path()).unwrap();
        assert_eq!(result["outcome"], "failed");
    }
}

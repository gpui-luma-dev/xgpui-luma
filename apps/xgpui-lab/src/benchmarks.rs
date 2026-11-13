//! Embedded Parley, Shader and SVG workloads. Isolated jobs launch only this executable.
pub(crate) mod memory;
pub(crate) mod progress;
pub(crate) mod svg_table;
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
        Job::SvgCorpus => match corpus {
            Some(path) => crate::svg_lab::benchmarks::corpus(path, output, cancel),
            None => crate::svg_lab::benchmarks::bundled_corpus(output, cancel),
        },
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
    pickers: Vec<Entity<Button>>,
    selected: Option<PathBuf>,
    corpus: Option<PathBuf>,
    cancel: Arc<AtomicBool>,
    busy: bool,
    svg_budget: usize,
    svg_budget_choice: crate::lab_controls::Choice,
    status: String,
    output: Option<PathBuf>,
    results: Vec<Value>,
    svg_table: xgpui_luma::controls::table::Table<svg_table::Row>,
    quit_on_finish: bool,
}
impl Drop for Page {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl Page {
    pub fn new(suite: Suite, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let buttons = std::iter::once((None, "Run all"))
            .chain(suite.jobs().into_iter().map(|job| (Some(job), job.label())))
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
            .label("Cancel")
            .on_click(cx, |this: &mut Self, cx| {
                this.cancel.store(true, Ordering::Relaxed);
                this.status = "Cancelling…".into();
                cx.notify();
            })
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
        let pickers = [false, true]
            .into_iter()
            .map(|directory| {
                radix::Button::new(format!("bench-picker-{directory}"))
                    .look(&look)
                    .outline()
                    .label(if directory {
                        "Override corpus folder…"
                    } else {
                        "Choose SVG file…"
                    })
                    .on_click(cx, move |this: &mut Self, cx| this.choose(directory, cx))
                    .spawn(cx)
            })
            .collect();
        let svg_budget_choice = crate::lab_controls::Choice::new(
            "svg-benchmark-budget",
            "Raster budget",
            &look,
            crate::lab_controls::labels(&["4 MiB", "16 MiB", "64 MiB", "256 MiB"]),
            0,
            cx,
            |this: &mut Self, index, cx| {
                this.svg_budget = [4, 16, 64, 256][index] * xgpui_svg::MIB;
                cx.notify();
            },
        );
        let svg_table = svg_table::build(&look, cx);
        Self {
            svg_table,
            suite,
            look,
            buttons,
            cancel_button,
            reveal,
            pickers,
            selected: None,
            corpus: None,
            cancel: Arc::new(AtomicBool::new(false)),
            busy: false,
            svg_budget: 4 * xgpui_svg::MIB,
            svg_budget_choice,
            status: if suite == Suite::Svg {
                "Choose a workload. Results stay in memory; interactive runs write no files."
            } else {
                "Choose a workload. Reports and captures use the embedded application code."
            }
            .into(),
            output: None,
            results: Vec::new(),
            quit_on_finish: false,
        }
    }
    fn choose(&mut self, directory: bool, cx: &mut Context<Self>) {
        let picker = cx.prompt_for_paths(PathPromptOptions {
            files: !directory,
            directories: directory,
            multiple: false,
            prompt: Some(
                if directory {
                    "Choose matching SVG corpus"
                } else {
                    "Choose SVG artwork"
                }
                .into(),
            ),
        });
        cx.spawn(async move |this, cx| {
            let selected = picker.await;
            let _ = this.update(cx, |this, cx| {
                match selected {
                    Ok(Ok(Some(paths))) => {
                        if let Some(path) = paths.into_iter().next() {
                            if directory {
                                this.corpus = Some(path);
                            } else {
                                this.selected = Some(path);
                            }
                        }
                    }
                    Ok(Ok(None)) => {}
                    _ => this.status = "File selection failed; previous selection retained.".into(),
                }
                cx.notify();
            });
        })
        .detach();
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
        self.svg_budget_choice.selector.update(cx, |selector, cx| selector.set_enabled(false, cx));
        for picker in &self.pickers {
            picker.update(cx, |button, cx| button.set_enabled(false, cx));
        }
        self.busy = true;
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
        let jobs = job.map_or_else(|| self.suite.jobs(), |job| vec![job]);
        let cancel = self.cancel.clone();
        let selected = self.selected.clone();
        let corpus = self.corpus.clone();
        let budget = self.svg_budget;
        let progress = progress::Progress::default();
        if in_memory {
            let progress = progress.clone();
            let token = cancel.clone();
            cx.spawn(async move |this, cx| {
                let mut last_count = 0;
                loop {
                    cx.background_executor().timer(Duration::from_millis(200)).await;
                    let (status, partial) = progress.snapshot();
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
                            if partial.len() != last_count {
                                let mut rows = svg_table::rows(&this.results);
                                rows.extend(svg_table::rows(&partial));
                                this.svg_table.update(cx, |table, cx| {
                                    let _ = table.set_items(rows, cx);
                                });
                                last_count = partial.len();
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
                let progress = progress.clone();
                let result = cx
                    .background_spawn(async move {
                        if let Some(directory) = directory {
                            let result = run_job(job, &directory, selected.as_deref(), corpus.as_deref(), &token);
                            std::fs::create_dir_all(&directory)?;
                            record(job, result, &directory)
                        } else {
                            Ok(decorate(
                                job,
                                run_svg_memory(job, selected.as_deref(), corpus.as_deref(), budget, &token, &progress),
                            ))
                        }
                    })
                    .await;
                if this
                    .update(cx, |this, cx| {
                        this.results.push(result.unwrap_or_else(
                            |error| json!({"label":job.label(),"outcome":"failed","error":format!("{error:#}")}),
                        ));
                        let rows = svg_table::rows(&this.results);
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
                this.svg_budget_choice.selector.update(cx, |selector, cx| selector.set_enabled(true, cx));
                for picker in &this.pickers {
                    picker.update(cx, |button, cx| button.set_enabled(true, cx));
                }
                if let Some(output) = &output
                    && let Err(error) = summary(output, &this.results, cancel.load(Ordering::Relaxed))
                {
                    this.results.push(json!({"outcome":"failed","error":format!("Cannot save summary: {error:#}")}));
                }
                let failed = this.results.iter().filter(|result| result["outcome"] == "failed").count();
                let unavailable = this.results.iter().filter(|result| result["outcome"] == "unavailable").count();
                this.status = format!(
                    "{} in {:.1}s: {} workloads, {failed} failures, {unavailable} unavailable.",
                    if cancel.load(Ordering::Relaxed) {
                        "Cancelled"
                    } else {
                        "Finished"
                    },
                    progress.elapsed(),
                    this.results.len()
                );
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
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        use crate::theme::ColorVizLookExt;
        div().id("embedded-benchmarks").size_full().overflow_y_scroll().p_4().flex().flex_col().gap_4().text_color(self.look.chrome().body_text)
            .child(format!("{:?} benchmarks · {} build",self.suite,profile()))
            .child(div().flex().flex_wrap().gap_2().children(self.buttons.clone()).when(self.busy,|row|row.child(self.cancel_button.clone())))
            .child(match self.suite{Suite::Parley=>"Both text backends use the same embedded samples and fonts. Scroll cases run 600 frames in a temporary window. Backend isolation uses this app’s executable.",Suite::Shader=>"Native pixel checks cover geometry, gradients, alpha, clipping, replay, resize, source reuse and portable WGSL. Timings include readback, not GPU-only time.",Suite::Svg=>"Five CPU parse/raster samples after first use. Selected SVG: 4× / density 2 / selected raster budget. Corpus: hash-verified files, four zooms, fresh application process per case. No artwork is downloaded."})
            .when(self.suite==Suite::Svg,|root|root.child(div().flex().gap_2().children(self.pickers.clone())).child(format!("SVG: {:?} · Corpus: {}",self.selected,self.corpus.as_ref().map_or_else(|| "Bundled samples (4 files)".to_owned(), |path| path.display().to_string()))))
            .when(self.suite==Suite::Svg,|root|root.child(self.svg_budget_choice.clone()).child("Corpus: 16 cases, 5 measured repeats plus 2 warm-up rasters per case. Budget caps the output image; SVG filter intermediates can require more memory. Larger budgets can take minutes."))
            .child(self.status.clone()).when_some(self.output.as_ref(),|root,output|root.child(format!("Results: {}",output.display())).child(self.reveal.clone()))
            .when(self.suite==Suite::Svg && (self.busy || !self.results.is_empty()),|root|root.child(self.svg_table.clone()))
            .children(self.results.iter().map(|result|{
                let mut row=div().flex().flex_col().gap_2().child(format!("{} · {}",result["label"].as_str().unwrap_or("Workload"),result["outcome"].as_str().unwrap_or("unknown")));
                for key in ["error","summary","measurement","stats"]{if let Some(text)=result[key].as_str(){row=row.child(text.to_owned());}}
                if let Some(errors)=result["errors"].as_array(){for error in errors{row=row.child(error.to_string());}}
                if let Some(ms)=result["raster"]["median_ms"].as_f64(){row=row.child(format!("Raster median: {ms:.3} ms · {}×{} pixels",result["pixel_width"],result["pixel_height"]));}
                if let Some(previews)=result["previews"].as_array(){for path in previews.iter().filter_map(Value::as_str){row=row.child(img(PathBuf::from(path)).max_w(px(760.)));}}row
            }))
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
    budget: usize,
    cancel: &AtomicBool,
    progress: &progress::Progress,
) -> Result<Value> {
    match job {
        Job::SvgBuiltin => crate::svg_lab::benchmarks::memory_selected(None, budget, cancel, progress),
        Job::SvgSelected => match selected {
            Some(path) => crate::svg_lab::benchmarks::memory_selected(Some(path), budget, cancel, progress),
            None => Ok(json!({"outcome":"unavailable","summary":"Choose an SVG file to run this workload."})),
        },
        Job::SvgCorpus => crate::svg_lab::benchmarks::memory_corpus(corpus, budget, cancel, progress),
        _ => anyhow::bail!("expected SVG workload"),
    }
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
    fn interactive_svg_run_keeps_numeric_results_in_memory() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| Page::new(Suite::Svg, crate::theme::default_look(), cx));
        page.update(cx, |page, cx| page.start(Some(Job::SvgBuiltin), None, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert!(!page.busy);
            assert_eq!(page.svg_budget, 4 * xgpui_svg::MIB);
            assert_eq!(page.results[0]["budget_bytes"], 4 * xgpui_svg::MIB);
            assert!(page.status.starts_with("Finished in "));
            assert!(page.output.is_none());
            assert_eq!(page.results.len(), 1);
            assert_eq!(page.results[0]["outcome"], "completed");
            assert!(page.results[0].get("previews").is_none());
            assert!(page.results[0]["raster"]["median_ms"].as_f64().is_some());
            assert_eq!(page.svg_table.read(cx).items().len(), 1);
        });
    }
    #[::core::prelude::v1::test]
    fn report_errors_are_failures_not_success() {
        let dir = tempfile::tempdir().unwrap();
        let result = record(Job::SvgCorpus, Ok(json!({"errors":["hash mismatch"]})), dir.path()).unwrap();
        assert_eq!(result["outcome"], "failed");
    }
}

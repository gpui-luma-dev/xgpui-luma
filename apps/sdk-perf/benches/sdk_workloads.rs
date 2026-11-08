//! Headless benchmark: never starts the native application platform.
use std::{io::Write, time::Duration};
use gpui::{prelude::*, AppContext, Context, Entity, Render, TestAppContext, Window, div, px, size};
use luma_sdk_perf::{LookKind, Scene, SceneKind};
use serde_json::json;
#[allow(dead_code)]
#[path = "../../../crates/sdk/src/perf_measure.rs"]
mod measurement;

struct Host {
    scene: Option<Entity<Scene>>,
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(1200.0)).h(px(800.0)).children(self.scene.iter().cloned())
    }
}

#[cfg(target_os = "macos")]
fn resident_bytes() -> Option<u64> {
    let mut info = std::mem::MaybeUninit::<libc::mach_task_basic_info>::zeroed();
    let mut count = libc::MACH_TASK_BASIC_INFO_COUNT;
    // SAFETY: The buffer and count describe exactly mach_task_basic_info. The
    // kernel initializes it only on KERN_SUCCESS; this queries our own task.
    #[allow(deprecated)]
    unsafe {
        let result =
            libc::task_info(libc::mach_task_self(), libc::MACH_TASK_BASIC_INFO, info.as_mut_ptr().cast(), &mut count);
        (result == libc::KERN_SUCCESS).then(|| info.assume_init().resident_size)
    }
}
#[cfg(not(target_os = "macos"))]
fn resident_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix("VmRSS:")
            .and_then(|value| value.split_whitespace().next()?.parse::<u64>().ok())
            .map(|kb| kb * 1024)
    })
}

fn record(out: &mut dyn Write, value: serde_json::Value) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *out, &value)?;
    writeln!(out)?;
    out.flush()?;
    Ok(())
}

fn run(kind: SceneKind, look: LookKind, count: usize, cycles: usize, out: &mut dyn Write) -> anyhow::Result<()> {
    let mut app = TestAppContext::single();
    let (host, cx) = app.add_window_view(|_, cx| Host {
        scene: Some(cx.new(|cx| Scene::new(kind, look, count, cx).expect("valid benchmark fixture"))),
    });
    cx.simulate_resize(size(px(1200.0), px(800.0)));
    anyhow::ensure!(
        cx.update(|window, _| window.viewport_size()) == size(px(1200.0), px(800.0)),
        "test viewport must match recorded dimensions"
    );
    cx.run_until_parked();
    let controls = cx.update(|_, app| host.read(app).scene.as_ref().unwrap().read(app).control_count());
    let has_updates = cx.update(|_, app| host.read(app).scene.as_ref().unwrap().read(app).has_updates());
    let iterations = if count >= 1000 { 2 } else { 5 };
    for phase in [
        "construct_drop",
        "replace_first_paint",
        "warm_redraw",
        "data_update",
        "theme_revision",
        "executor_idle_drain",
    ] {
        if phase == "data_update" && !has_updates {
            continue;
        }
        let before_renders = cx.update(|_, app| host.read(app).scene.as_ref().unwrap().read(app).render_count());
        let mut step = 0;
        let stats = measurement::sample(iterations, || {
            step += 1;
            if phase == "construct_drop" {
                cx.update(|_, app| {
                    drop(app.new(|cx| Scene::new(kind, look, count, cx).expect("valid fixture")));
                });
                cx.run_until_parked();
                return;
            }
            if phase == "replace_first_paint" {
                host.update(cx, |host, cx| {
                    host.scene = Some(cx.new(|cx| Scene::new(kind, look, count, cx).expect("valid fixture")));
                    cx.notify();
                });
            } else if phase != "executor_idle_drain" {
                cx.update(|_, app| {
                    let scene = host.read(app).scene.as_ref().unwrap().clone();
                    match phase {
                        "warm_redraw" => scene.update(app, |scene, cx| {
                            scene.invalidate(cx);
                            cx.notify();
                        }),
                        "data_update" => scene.update(app, |scene, cx| {
                            scene.update_data(step, cx);
                            cx.notify();
                        }),
                        "theme_revision" => Scene::invalidate_theme(app),
                        _ => {}
                    }
                });
            }
            cx.run_until_parked();
        });
        let after_renders = cx.update(|_, app| host.read(app).scene.as_ref().unwrap().read(app).render_count());
        if phase == "warm_redraw" {
            anyhow::ensure!(after_renders > before_renders, "redraw workload must actually render");
        }
        record(
            out,
            json!({"type":"measurement", "scene":kind.name(),"look":look.name(),"size":count,"controls":controls,"phase":phase,"iterations":iterations,"batch_mean_ns":stats.samples_ns,"median_ns":stats.median_ns,"min_ns":stats.min_ns,"max_ns":stats.max_ns,"allocations_per_op":stats.allocations,"requested_bytes_per_op":stats.requested_bytes,"resident_bytes":resident_bytes()}),
        )?;
    }
    let probes = cx.update(|_, app| host.read(app).scene.as_ref().unwrap().read(app).release_probes());
    host.update(cx, |host, cx| {
        host.scene = None;
        cx.notify();
    });
    cx.run_until_parked();
    // Flush both frame buffers before testing retained entity ownership.
    for _ in 0..2 {
        cx.update(|window, app| window.simulate_next_frame(app));
        cx.run_until_parked();
    }
    let released = probes.iter().filter(|probe| probe()).count();
    record(
        out,
        json!({"type":"release_check","scene":kind.name(),"look":look.name(),"size":count,"tracked":probes.len(),"released":released,"resident_bytes":resident_bytes()}),
    )?;
    anyhow::ensure!(released == probes.len(), "fixture entities remain alive after teardown");
    // Repeated lifecycles reveal process memory trends, including allocator and
    // GPUI caches. They cannot distinguish retained memory from a leak alone.
    let mut checkpoints = vec![1, 10, cycles];
    checkpoints.sort_unstable();
    checkpoints.dedup();
    let mut completed = 0;
    for checkpoint in checkpoints.into_iter().filter(|checkpoint| *checkpoint > 0 && *checkpoint <= cycles) {
        for _ in completed..checkpoint {
            host.update(cx, |host, cx| {
                host.scene = Some(cx.new(|cx| Scene::new(kind, look, count, cx).expect("valid fixture")));
                cx.notify();
            });
            cx.run_until_parked();
            host.update(cx, |host, cx| {
                host.scene = None;
                cx.notify();
            });
            cx.run_until_parked();
        }
        completed = checkpoint;
        record(
            out,
            json!({"type":"lifecycle_memory","scene":kind.name(),"look":look.name(),"size":count,"completed_cycles":checkpoint,"resident_bytes":resident_bytes()}),
        )?;
    }
    // A bounded idle interval with no requested animation is not native idle CPU.
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    // add_window_view registers its context for explicit on_quit cleanup.
    app.quit();
    app.run_until_parked();
    Ok(())
}

fn infrastructure(out: &mut dyn Write) -> anyhow::Result<()> {
    use std::hint::black_box;
    use xgpui_luma::{
        theme::{ControlSize, MetricTokens, StandardBoxScale, ThemeTokens},
        motion::{MotionState, Transition},
    };
    let metrics = MetricTokens::default();
    let tokens = ThemeTokens::default();
    for name in ["layout_scale", "theme_tokens_clone", "motion_start_and_sample"] {
        let stats = measurement::sample(10_000, || match name {
            "layout_scale" => {
                black_box(StandardBoxScale::compute(black_box(ControlSize::Md), black_box(&metrics), black_box(2.0)));
            }
            "theme_tokens_clone" => {
                black_box(black_box(&tokens).clone());
            }
            _ => {
                let transition = Transition::spring(300.0, 28.0);
                let motion = transition.start(black_box(MotionState::default()), black_box(1.0));
                for elapsed in [0, 16, 160, 1000] {
                    black_box(motion.sample(Duration::from_millis(elapsed)));
                }
            }
        });
        record(
            out,
            json!({"type":"infrastructure","workload":name,"iterations":10000,"batch_mean_ns":stats.samples_ns,"median_ns":stats.median_ns,"min_ns":stats.min_ns,"max_ns":stats.max_ns,"allocations_per_op":stats.allocations,"requested_bytes_per_op":stats.requested_bytes}),
        )?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.iter().any(|arg| arg == "--measure") {
        return Ok(());
    }
    let mut options = std::collections::HashMap::new();
    let mut arguments = args.iter();
    while let Some(argument) = arguments.next() {
        if argument == "--measure" || argument == "--bench" {
            continue;
        }
        anyhow::ensure!(
            matches!(argument.as_str(), "--scene" | "--look" | "--size" | "--cycles" | "--output"),
            "unknown option: {argument}"
        );
        let value = arguments.next().ok_or_else(|| anyhow::anyhow!("missing value for {argument}"))?;
        anyhow::ensure!(!value.starts_with("--"), "missing value for {argument}");
        anyhow::ensure!(options.insert(argument.as_str(), value.as_str()).is_none(), "duplicate option: {argument}");
    }
    let option = |name: &str| options.get(name).copied();
    let cycles = option("--cycles").map(str::parse::<usize>).transpose()?.unwrap_or(10);
    anyhow::ensure!(cycles <= 1000, "cycles must be 0..1000");
    let kind = option("--scene")
        .map(|value| SceneKind::parse(value).ok_or_else(|| anyhow::anyhow!("unknown scene")))
        .transpose()?;
    let look = option("--look")
        .map(|value| LookKind::parse(value).ok_or_else(|| anyhow::anyhow!("unknown look")))
        .transpose()?;
    let requested_size = option("--size").map(str::parse::<usize>).transpose()?;
    if let Some(size) = requested_size {
        anyhow::ensure!(size > 0 && size <= 100_000, "size must be 1..100000");
    }
    anyhow::ensure!(
        !(kind.is_some_and(|kind| kind != SceneKind::Forms) && look.is_some_and(|look| look != LookKind::Sdk)),
        "only forms compare look adapters"
    );
    let mut out: Box<dyn Write> = match option("--output") {
        Some(path) => Box::new(std::fs::File::create(path)?),
        None => Box::new(std::io::stdout()),
    };
    let command = |program: &str, args: &[&str]| {
        std::process::Command::new(program)
            .args(args)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    };
    record(
        &mut out,
        json!({"type":"environment","schema_version":1,"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"rustc":command("rustc", &["-Vv"]),"commit":command("git", &["rev-parse","HEAD"]),"dirty":command("git", &["status","--porcelain"]),"profile":"bench/release","debug_assertions":cfg!(debug_assertions),"captured_unix_seconds":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok().map(|duration| duration.as_secs()),"viewport":[1200,800],"allocation_scope":"measuring thread; calls and requested bytes, not retained memory","latency_scope":"headless test platform CPU; not native frame or input latency"}),
    )?;
    if kind.is_none() && look.is_none() {
        infrastructure(&mut out)?;
    }
    let mut fixtures = 0;
    for scene in SceneKind::ALL {
        if kind.is_some_and(|kind| kind != scene) {
            continue;
        }
        for adapter in LookKind::ALL {
            if look.is_some_and(|look| look != adapter) || (scene != SceneKind::Forms && adapter != LookKind::Sdk) {
                continue;
            }
            let sizes: Vec<usize> = match requested_size {
                Some(size) => vec![size],
                None => match scene {
                    SceneKind::Forms | SceneKind::Layout => vec![10, 100, 1000],
                    SceneKind::Navigation | SceneKind::Collections => vec![100, 10000],
                    SceneKind::Color => vec![1, 10, 100],
                    SceneKind::Plot => vec![100, 10000, 100000],
                },
            };
            for count in sizes {
                eprintln!("measuring {}/{}/{count}", scene.name(), adapter.name());
                run(scene, adapter, count, cycles, &mut out)?;
                fixtures += 1;
            }
        }
    }
    record(&mut out, json!({"type":"complete","fixtures":fixtures}))?;
    Ok(())
}

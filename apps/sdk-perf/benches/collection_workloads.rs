//! Interaction-focused collection measurements on GPUI's headless test platform.
use std::io::Write;
use gpui::TestAppContext;
use serde_json::json;
#[allow(dead_code)]
#[path = "../../../crates/sdk/src/perf_measure.rs"]
mod measurement;
#[path = "support/collections.rs"]
mod collections;
use collections::*;

#[cfg(target_os = "macos")]
fn rss() -> Option<u64> {
    let mut info = std::mem::MaybeUninit::<libc::mach_task_basic_info>::zeroed();
    let mut count = libc::MACH_TASK_BASIC_INFO_COUNT;
    // SAFETY: task_info receives a correctly sized writable buffer for our task;
    // assume_init is used only after the kernel reports success.
    unsafe {
        let status =
            libc::task_info(libc::mach_task_self(), libc::MACH_TASK_BASIC_INFO, info.as_mut_ptr().cast(), &mut count);
        (status == libc::KERN_SUCCESS).then(|| info.assume_init().resident_size)
    }
}
#[cfg(not(target_os = "macos"))]
fn rss() -> Option<u64> {
    None
}
fn record(out: &mut dyn Write, value: serde_json::Value) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *out, &value)?;
    writeln!(out)?;
    out.flush()?;
    Ok(())
}
fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !args.iter().any(|a| a == "--measure") {
        return Ok(());
    }
    let mut sizes = vec![100, 10_000];
    let mut cases = Case::ALL.to_vec();
    let mut output = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--measure" || arg == "--bench" {
            continue;
        }
        let value = args.next().ok_or_else(|| anyhow::anyhow!("missing value for {arg}"))?;
        match arg.as_str() {
            "--size" => {
                let n = value.parse::<usize>()?;
                anyhow::ensure!((100..=100_000).contains(&n), "size must be 100..100000");
                sizes = vec![n];
            }
            "--case" => {
                cases =
                    vec![*Case::ALL.iter().find(|c| c.name() == value).ok_or_else(|| anyhow::anyhow!("unknown case"))?]
            }
            "--output" => output = Some(value.clone()),
            _ => anyhow::bail!("unknown option: {arg}"),
        }
    }
    let mut out: Box<dyn Write> = match output {
        Some(path) => Box::new(std::fs::File::create(path)?),
        None => Box::new(std::io::stdout()),
    };
    let command = |program: &str, args: &[&str]| {
        std::process::Command::new(program)
            .args(args)
            .output()
            .ok()
            .filter(|r| r.status.success())
            .map(|r| String::from_utf8_lossy(&r.stdout).trim().to_owned())
    };
    record(
        &mut out,
        json!({"type":"environment", "schema_version":2, "os":std::env::consts::OS,"arch":std::env::consts::ARCH,"rustc":command("rustc", &["--version"]),"commit":command("git", &["rev-parse","HEAD"]),"dirty":command("git", &["status","--porcelain"]),"captured_unix_seconds":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),"viewport":[1100,700],"table_default_distant_reveal_verified":true,"cases":cases.iter().map(|case|case.name()).collect::<Vec<_>>(),"sizes":sizes,"debug_assertions":cfg!(debug_assertions),"scope":"headless mock fonts/graphics; CPU and measuring-thread allocation traffic; not native input latency"}),
    )?;
    let mut measurements = 0;
    for count in sizes {
        for case in &cases {
            for phase in case.phases() {
                if *case == Case::TreeClosed
                    && ((count < 202 && matches!(phase, "keyboard" | "selection"))
                        || (count.div_ceil(101) <= 22 && phase == "wheel"))
                {
                    continue;
                }
                eprintln!("{}/{count}/{phase}", case.name());
                let mut app = TestAppContext::single();
                let app_probe = std::rc::Rc::downgrade(&app.app);
                let (host, cx, paints) = setup(&mut app, *case, count);
                let before = paints.calls();
                let mut step = 0;
                let iterations = if count >= 10_000 { 2 } else { 5 };
                let stats = measurement::sample(iterations, || {
                    step += 1;
                    if phase == "construct_drop" || phase == "first_paint" {
                        cx.update(|_, app| {
                            let next = control(*case, count, paints.clone(), app);
                            if phase == "first_paint" {
                                host.update(app, |host, cx| {
                                    host.control = Some(next);
                                    cx.notify();
                                });
                            }
                        });
                        cx.run_until_parked();
                    } else {
                        operate(&host, cx, phase, count, step);
                    }
                });
                let painted = paints.calls() - before;
                if matches!(phase, "warm_redraw" | "wheel" | "column_resize") {
                    anyhow::ensure!(painted > 0, "{phase} must paint rows");
                }
                let resident = rss();
                anyhow::ensure!(released(&host, cx), "primary entity retained after teardown");
                let after_release = rss();
                drop(host);
                // add_window_view registers its VisualTestContext for on_quit
                // cleanup. Dropping TestAppContext alone retains that context.
                app.quit();
                app.run_until_parked();
                drop(app);
                anyhow::ensure!(app_probe.upgrade().is_none(), "test application retained after shutdown");
                record(
                    &mut out,
                    json!({"type":"measurement","case":case.name(),"size":count,"phase":phase,"iterations":iterations,"batch_mean_ns":stats.samples_ns,"median_ns":stats.median_ns,"min_ns":stats.min_ns,"max_ns":stats.max_ns,"allocations_per_op":stats.allocations,"requested_bytes_per_op":stats.requested_bytes,"presenter_calls_initial_setup":before,"presenter_calls_all_passes":painted,"operation_calls_all_passes":step,"resident_bytes_before_release":resident,"resident_bytes_after_release":after_release,"resident_bytes_after_app_shutdown":rss(),"released":true,"app_released":true}),
                )?;
                measurements += 1;
                anyhow::ensure!(
                    rss().is_none_or(|bytes| bytes < 8 * 1024 * 1024 * 1024),
                    "process RSS exceeded 8 GiB after shutdown; rerun a smaller isolated case"
                );
            }
        }
    }
    record(&mut out, json!({"type":"complete","measurements":measurements}))
}

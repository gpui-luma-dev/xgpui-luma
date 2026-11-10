#![windows_subsystem = "windows"]

mod demo_check;
mod canvas_lab;
mod pastel_waves;
mod standalone_benchmarks;
mod app;
mod chromaticity;
mod color_gamut_page;
mod lab_controls;
mod screen;
mod benchmarks;

mod app_shell;
mod shell_lab;
mod compositions;
mod extension_pane;
mod gamut;
mod color_lab;
mod color_stop;
mod color_stop_list;
mod color_demos;
mod media_lab;
mod photo_viewport;
mod parley_lab;
mod svg_lab;
mod shader;
mod gradient_builder;
mod theme;
mod studio_tabs;
#[path = "assets/assets.rs"]
mod assets;

use assets::Assets;
use gpui::{actions, App, KeyBinding, Menu, MenuItem};

actions!(xgpui_lab_app, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    // This path also verifies that a packaged Windows executable can load without
    // opening a GPU window or accessing a build machine's source directories.
    if std::env::args().any(|arg| arg == "--version") {
        println!(
            "XGPUI Lab {} ({}-{}, {})",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::ARCH,
            std::env::consts::OS,
            if cfg!(feature = "portable-app") {
                "portable"
            } else {
                "source"
            }
        );
        return;
    }
    let args: Vec<String> = std::env::args().collect();
    if let Some(output) = benchmarks::argument(&args, "--waves-check") {
        let output = std::path::PathBuf::from(output);
        gpui_platform::application().with_assets(Assets).run(move |cx| {
            if let Err(error) = pastel_waves::run_check(cx, output) {
                eprintln!("Waves check: {error:#}");
                std::process::exit(1);
            }
        });
        return;
    }
    if let Some(output) = benchmarks::argument(&args, "--demo-check") {
        let output = std::path::PathBuf::from(output);
        gpui_platform::application().with_assets(Assets).run(move |cx| {
            if let Err(error) = demo_check::run(cx, output) {
                eprintln!("Demo check: {error:#}");
                std::process::exit(1);
            }
        });
        return;
    }
    if let Some(backend) = benchmarks::argument(&args, "--text-benchmark-backend") {
        let result = (|| -> anyhow::Result<()> {
            parley_lab::native::benchmark(
                backend,
                benchmarks::argument(&args, "--text-benchmark-mode").unwrap_or("cpu"),
                std::path::Path::new(
                    benchmarks::argument(&args, "--output")
                        .ok_or_else(|| anyhow::anyhow!("output directory required"))?,
                ),
            )
        })();
        if let Err(error) = result {
            eprintln!("Text benchmark: {error:#}");
            std::process::exit(1);
        }
        return;
    }
    if args.iter().any(|arg| arg == "--svg-case") {
        if let Err(error) = svg_lab::benchmarks::run_case_cli(&args) {
            eprintln!("SVG benchmark: {error:#}");
            std::process::exit(1);
        }
        return;
    }
    if args.iter().any(|arg| matches!(arg.as_str(), "--standalone-benchmark" | "--dense-producer-worker")) {
        if let Err(error) = standalone_benchmarks::run_cli(&args) {
            eprintln!("Standalone benchmark: {error:#}");
            std::process::exit(1);
        }
        return;
    }
    for (flag, suite) in [
        ("--parley-benchmark", benchmarks::Suite::Parley),
        ("--shader-benchmark", benchmarks::Suite::Shader),
        ("--svg-benchmark", benchmarks::Suite::Svg),
    ] {
        if args.iter().any(|arg| arg == flag) {
            if let Err(error) = benchmarks::run_cli(&args, suite, flag) {
                eprintln!("Benchmark: {error:#}");
                std::process::exit(1);
            }
            return;
        }
    }
    if args.iter().any(|arg| arg == "--media-benchmark") {
        if let Err(error) = media_lab::benchmarks::run_cli(&args) {
            eprintln!("Media benchmark: {error:#}");
            std::process::exit(1);
        }
        return;
    }
    if args.iter().any(|arg| arg == "--gamut-benchmark") {
        if let Err(error) = gamut::benchmarks::run_cli(&args) {
            eprintln!("Gamut benchmark: {error:#}");
            std::process::exit(1);
        }
        return;
    }
    if let Some(backend) =
        std::env::args().find_map(|arg| arg.strip_prefix("--native-text-backend=").map(str::to_owned))
    {
        let frames = std::env::args()
            .find_map(|arg| arg.strip_prefix("--scroll-frames=").and_then(|value| value.parse::<usize>().ok()))
            .unwrap_or(0);
        if let Err(error) = parley_lab::native::run(&backend, frames) {
            eprintln!("Text diagnostic failed: {error:#}");
            std::process::exit(1);
        }
        return;
    }
    if let Some(output) = benchmarks::argument(&args, "--photo-check") {
        let output = std::path::PathBuf::from(output);
        gpui_platform::application().with_assets(Assets).run(move |cx| {
            if let Err(error) = xgpui_luma::init(cx).and_then(|_| photo_viewport::run_check(cx, output)) {
                eprintln!("Photo check: {error:#}");
                std::process::exit(1);
            }
        });
        return;
    }
    // `--text-system=parley` runs the whole lab on the Parley text system, so
    // pages that edit through GPUI's paragraph API (the Parley Text Field) work.
    let parley_text_system = std::env::args()
        .any(|arg| arg == "--text-system=parley")
        .then(|| {
            parley_lab::application_text_system()
                .inspect_err(|error| eprintln!("Parley text system unavailable, using the native one: {error:#}"))
                .ok()
        })
        .flatten();
    let parley = parley_text_system.is_some();
    let app = match parley_text_system {
        Some(text_system) => {
            gpui::Application::with_platform_and_text_system(gpui_platform::current_platform(false), text_system)
        }
        None => gpui_platform::application(),
    }
    .with_assets(Assets);

    app.run(move |cx| {
        if parley {
            cx.set_global(parley_lab::ParleyTextSystemActive);
        }
        cx.on_action(quit);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus([Menu::new("XGPUI Lab").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = xgpui_luma::init(cx).and_then(|_| {
            theme::register_fonts(cx)?;
            xgpui_luma::focus::bind_default_focus_keys(cx);
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            app_shell::open(cx)
        }) {
            eprintln!("failed to open XGPUI Lab: {error:?}");
        }
    });
}

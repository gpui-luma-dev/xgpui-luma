//! Optional native profiling fixture. Building this target does not launch it.
use gpui::{AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use luma_sdk_perf::{LookKind, Scene, SceneKind};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    anyhow::ensure!(
        args.len() == 3 || (args.len() == 4 && matches!(args[3].as_str(), "--drive" | "--idle")),
        "usage: luma-sdk-perf <forms|navigation|collections|layout|color|plot> <sdk|shadcn|radix> <size> [--drive|--idle]"
    );
    let driven = args.get(3).is_some_and(|arg| arg == "--drive");
    let idle = args.get(3).is_some_and(|arg| arg == "--idle");
    let kind = SceneKind::parse(&args[0]).ok_or_else(|| anyhow::anyhow!("unknown scene"))?;
    let look = LookKind::parse(&args[1]).ok_or_else(|| anyhow::anyhow!("unknown look"))?;
    let count: usize = args[2].parse()?;
    anyhow::ensure!(count > 0 && count <= 100_000, "size must be 1..100000");
    anyhow::ensure!(kind == SceneKind::Forms || look == LookKind::Sdk, "only forms compare look adapters");
    gpui_platform::application().with_assets(Assets).run(move |cx| {
        let result = xgpui_luma::init(cx).and_then(|_| {
            luma_app_common::register_all(cx)?;
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
            let handle = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    #[cfg(target_os = "macos")]
                    window_background: gpui::WindowBackgroundAppearance::Blurred,
                    ..Default::default()
                },
                |_, cx| {
                    // Inputs above are validated; dataset IDs in the fixture are fixed.
                    cx.new(|cx| Scene::new(kind, look, count, cx).expect("validated profiling fixture"))
                },
            )?;
            let start = std::time::Instant::now();
            let frames = std::rc::Rc::new(std::cell::Cell::new(0));
            if driven {
                handle.update(cx, |_, window, cx| {
                    let weak = cx.entity().downgrade();
                    drive(window, weak, frames.clone());
                })?;
            }
            if driven || idle {
                cx.spawn(async move |cx| {
                    cx.background_executor().timer(std::time::Duration::from_secs(12)).await;
                    let _ = cx.update(|cx| {
                        if driven {
                            eprintln!("native workload: {} frame callbacks in {:?}; callbacks are not GPU presentation timestamps", frames.get(), start.elapsed());
                        } else {
                            eprintln!("native idle workload completed after 12 seconds");
                        }
                        cx.quit();
                    });
                })
                .detach();
            }
            cx.activate(true);
            Ok(())
        });
        if let Err(error) = result {
            eprintln!("could not open profiling fixture: {error:#}");
            cx.quit();
        }
    });
    Ok(())
}

struct Assets;
impl gpui::AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Ok(lucide_svg_static::asset_bytes(path).map(std::borrow::Cow::Borrowed))
    }
    fn list(&self, _: &str) -> anyhow::Result<Vec<gpui::SharedString>> {
        Ok(Vec::new())
    }
}

// Opt-in stress workload: continuously mutate and invalidate the same scene for
// twelve seconds. Callback cadence is not GPU presentation or input latency.
fn drive(window: &mut gpui::Window, scene: gpui::WeakEntity<Scene>, frames: std::rc::Rc<std::cell::Cell<usize>>) {
    window.on_next_frame(move |window, cx| {
        let step = frames.get();
        frames.set(step + 1);
        if let Some(entity) = scene.upgrade() {
            entity.update(cx, |scene, cx| {
                scene.update_data(step, cx);
                scene.invalidate(cx);
                cx.notify();
            });
            drive(window, scene, frames);
        }
    });
}

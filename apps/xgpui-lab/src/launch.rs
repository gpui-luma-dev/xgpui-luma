//! Command-line launch options for the normal Lab window:
//!
//! ```text
//! xgpui-lab --open "xgpui-media/Image Filters"        start on a page
//! xgpui-lab --open Typography --theme light           unique page names work alone
//! xgpui-lab --open Typography --hide-nav              start with the navigation panel closed
//! xgpui-lab --open "Color Spaces" --size 1280x900 --capture out.png [--settle 3000]
//! xgpui-lab --list-pages                              print every page
//! ```
//!
//! `--capture` opens the real Lab window at the page, waits for it to settle,
//! saves the window's rendered frame as a PNG and quits. `scripts/lab_snapshots.py`
//! uses it for before/after regression snapshots.
use std::{path::PathBuf, time::Duration};

use anyhow::{Context as _, Result, anyhow, bail, ensure};
use gpui::{App, AppContext, Global, Pixels, Size, px, size};
use xgpui_luma::theme::ThemeMode;

use crate::app::Route;

pub(crate) struct LaunchOptions {
    pub route: Option<Route>,
    pub theme: Option<ThemeMode>,
    pub size: Option<Size<Pixels>>,
    /// `--hide-nav`: start with the navigation panel closed (the titlebar toggle reopens it).
    pub hide_nav: bool,
    pub capture: Option<PathBuf>,
    pub settle: Duration,
}

impl Global for LaunchOptions {}

const DEFAULT_SETTLE: Duration = Duration::from_millis(3000);

fn value<'a>(args: &'a [String], flag: &str) -> Result<Option<&'a str>> {
    match args.iter().position(|arg| arg == flag) {
        None => Ok(None),
        Some(index) => {
            args.get(index + 1).map(|value| Some(value.as_str())).ok_or_else(|| anyhow!("{flag} needs a value"))
        }
    }
}

impl LaunchOptions {
    /// `None` when no launch option is present, so a plain start is unchanged.
    pub(crate) fn parse(args: &[String]) -> Result<Option<Self>> {
        let route = value(args, "--open")?.map(Route::resolve).transpose().map_err(|error| anyhow!(error))?;
        let theme = value(args, "--theme")?
            .map(|theme| match theme.to_ascii_lowercase().as_str() {
                "light" => Ok(ThemeMode::Light),
                "dark" => Ok(ThemeMode::Dark),
                other => Err(anyhow!("--theme must be light or dark, not {other:?}")),
            })
            .transpose()?;
        let size = value(args, "--size")?
            .map(|spec| -> Result<Size<Pixels>> {
                let (width, height) =
                    spec.split_once('x').ok_or_else(|| anyhow!("--size is WIDTHxHEIGHT, e.g. 1280x900"))?;
                let width: f32 = width.trim().parse().context("--size width")?;
                let height: f32 = height.trim().parse().context("--size height")?;
                ensure!(width >= 720. && height >= 480., "--size is at least 720x480 (the window minimum)");
                Ok(size(px(width), px(height)))
            })
            .transpose()?;
        let capture = value(args, "--capture")?.map(PathBuf::from);
        let hide_nav = args.iter().any(|arg| arg == "--hide-nav");
        let settle = value(args, "--settle")?
            .map(|ms| ms.parse::<u64>().map(Duration::from_millis).context("--settle is milliseconds"))
            .transpose()?
            .unwrap_or(DEFAULT_SETTLE);
        if capture.is_some() && route.is_none() {
            bail!("--capture needs --open <page>");
        }
        Ok(
            (route.is_some() || theme.is_some() || size.is_some() || capture.is_some() || hide_nav).then_some(Self {
                route,
                theme,
                size,
                hide_nav,
                capture,
                settle,
            }),
        )
    }
}

/// After the window opens: wait for the page to settle, save its frame, quit.
pub(crate) fn schedule_capture(cx: &mut App) {
    let Some((path, settle)) = cx
        .try_global::<LaunchOptions>()
        .and_then(|launch| launch.capture.clone().map(|path| (path, launch.settle)))
    else {
        return;
    };
    cx.spawn(async move |cx| {
        cx.background_executor().timer(settle).await;
        let result = cx.update(|cx| capture(cx, &path));
        match result {
            Ok(()) => eprintln!("saved {}", path.display()),
            Err(error) => {
                eprintln!("capture failed: {error:#}");
                std::process::exit(1);
            }
        }
        cx.update(|cx| cx.quit());
    })
    .detach();
}

fn capture(cx: &mut App, path: &std::path::Path) -> Result<()> {
    let window = cx.windows().into_iter().next().ok_or_else(|| anyhow!("no Lab window to capture"))?;
    cx.update_window(window, |_, window, cx| -> Result<()> {
        // Draw a fresh frame so the capture reflects the settled page, not an older frame.
        window.refresh();
        window.draw(cx).clear(cx);
        let image = window.render_to_image().context("window readback")?;
        ensure!(image.width() > 0 && image.height() > 0, "empty capture");
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        image.save(path)?;
        Ok(())
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        std::iter::once("xgpui-lab").chain(list.iter().copied()).map(str::to_owned).collect()
    }

    #[test]
    fn routes_resolve_by_section_and_page_or_unique_name() {
        let filters = Route::resolve("xgpui-media/Image Filters").unwrap();
        assert_eq!(Route::resolve("image filters").unwrap(), filters, "case-insensitive unique name");
        assert!(Route::resolve("Typography").is_ok());
        assert_eq!(Route::resolve("xgpui-media/Float / P3").unwrap(), Route::resolve("Float / P3").unwrap());
        // "Gamut Mapping" exists in xgpui-color and xgpui-gamut, so it needs a section.
        let ambiguous = Route::resolve("Gamut Mapping").unwrap_err();
        assert!(ambiguous.contains("xgpui-color/Gamut Mapping") && ambiguous.contains("xgpui-gamut/Gamut Mapping"));
        assert_ne!(
            Route::resolve("xgpui-color/Gamut Mapping").unwrap(),
            Route::resolve("xgpui-gamut/Gamut Mapping").unwrap()
        );
        assert!(Route::resolve("No Such Page").unwrap_err().contains("--list-pages"));
        assert!(Route::all().contains(&"xgpui-shader/Typography".to_owned()));
    }

    #[test]
    fn launch_options_parse_and_validate() {
        assert!(LaunchOptions::parse(&args(&[])).unwrap().is_none(), "a plain start is unchanged");
        let launch = LaunchOptions::parse(&args(&[
            "--open",
            "Color Spaces",
            "--theme",
            "light",
            "--size",
            "1280x900",
            "--capture",
            "out.png",
            "--settle",
            "500",
            "--hide-nav",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(launch.theme, Some(ThemeMode::Light));
        assert_eq!(launch.size, Some(size(px(1280.), px(900.))));
        assert_eq!(launch.capture, Some(PathBuf::from("out.png")));
        assert_eq!(launch.settle, Duration::from_millis(500));
        assert!(launch.hide_nav);
        assert!(LaunchOptions::parse(&args(&["--hide-nav"])).unwrap().unwrap().hide_nav, "works alone");
        for bad in [&["--capture", "out.png"][..], &["--theme", "sepia"], &["--size", "100x100"], &["--open"]] {
            assert!(LaunchOptions::parse(&args(bad)).is_err(), "{bad:?} is rejected");
        }
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn the_lab_opens_on_the_requested_page_and_theme() {
        let mut app = gpui::TestAppContext::single();
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            cx.set_global(LaunchOptions::parse(&args(&["--open", "Typography", "--theme", "light"])).unwrap().unwrap());
        });
        let (view, cx) = app.add_window_view(crate::app::XgpuiLabApp::new);
        cx.run_until_parked();
        cx.update(|_, cx| {
            let lab = view.read(cx);
            assert_eq!(lab.route(), Route::resolve("xgpui-shader/Typography").unwrap());
            assert_eq!(lab.look_mode(), ThemeMode::Light);
        });
    }
}

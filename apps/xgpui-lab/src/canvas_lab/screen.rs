//! Spatial Canvas and Mandelbrot, laid out with `LabScreen`. One `Lab` hosts both;
//! `fractal.enabled` picks the page.
use lucide_svg_static::Icon;
use xgpui_luma::controls::{button::Button, switch::Switch};
use xgpui_luma_look_radix::{self as radix, Badge, BadgeVariant};

use super::*;
use crate::lab_controls::{Choice, labels};
use crate::screen::{Hero, LabCard, LabScreen, labeled_control};

/// Height of the canvas viewport. Wide enough monitors show the hero, caption and controls above the fold.
pub(super) const CANVAS_HEIGHT: f32 = 560.;
const SCENE_SIZES: [usize; 2] = [10_000, 100_000];
const SWEEP_FRAMES: usize = 450;

const CANVAS_NOTE: &str = "Drag the background or Space-drag to pan, and scroll or pinch to zoom. Shift-drag selects; drag cards to move them and snap to their neighbors (Option moves freely); Esc cancels. Keys: G grid · R reset · O overview · B 10k / 100k · M map · T map mode.";
const CANVAS_CREDITS: &str = "Cards too small for full detail draw as filled proxies, then as 1 px points. The current-view map copies the drawn canvas, which adds capture cost. The sweep records render-callback intervals over 450 frames, not GPU time or displayed FPS; its captures and timing JSON go to a temporary folder.";
const FRACTAL_NOTE: &str = "Drag to pan, and scroll or pinch to zoom up to 10¹⁰×. The shown frame stays whole while its successor renders offscreen; a settled view refines and blends in. Keys: R reset · M preview.";
const FRACTAL_CREDITS: &str = "The Metal route computes in f32 and is used only up to 64× zoom; deeper views use f64 on the CPU, without perturbation. Colors are a sine palette, linearized and shown in Display P3. The preview copies the drawn frame, which adds capture cost.";

/// Counts of what the canvas drew this frame.
#[derive(Clone, Copy, Default)]
pub(super) struct Drawn {
    pub cards: usize,
    pub proxies: usize,
    pub points: usize,
}

/// The Lab's controls. Each one's look and callback are declared together in `Controls::new`.
pub(super) struct Controls {
    pub scene: Choice,
    grid: Choice,
    map: Choice,
    renderer: Choice,
    preview: Switch,
    pub reset: Entity<Button>,
    overview: Entity<Button>,
    run: Entity<Button>,
    cancel: Entity<Button>,
    reveal: Entity<Button>,
}

impl Controls {
    pub fn new(look: &Look, cx: &mut Context<Lab>) -> Self {
        use xgpui_luma::prelude::HasPresenter;
        let mut button = |id: &str, label: &str, action: fn(&mut Lab, &mut Context<Lab>)| {
            radix::Button::new(format!("canvas-{id}"))
                .look(look)
                .outline()
                .label(label)
                .on_click(cx, move |lab: &mut Lab, cx| {
                    lab.cancel_interaction();
                    action(lab, cx);
                    cx.notify();
                })
                .spawn(cx)
        };
        let reset = button("reset", "Reset view", |lab, _| lab.reset_view());
        let overview = button("overview", "Overview", |lab, _| lab.overview_zoom());
        let run = button("run-sweep", "Run sweep", |lab, _| lab.run_sweep());
        let cancel = button("cancel-sweep", "Cancel sweep", |lab, _| {
            lab.profile = None;
            lab.status = Some("Sweep cancelled; partial captures retained.".into());
        });
        let reveal = button("reveal", "Reveal results", |lab, cx| {
            if let Some(folder) = &lab.results {
                cx.reveal_path(folder);
            }
        });
        let choice = |id: &str,
                      label: &str,
                      options: &[&str],
                      cx: &mut Context<Lab>,
                      on_change: fn(&mut Lab, usize)| {
            Choice::new(format!("canvas-{id}"), label, look, labels(options), 0, cx, move |lab: &mut Lab, index, cx| {
                lab.cancel_interaction();
                on_change(lab, index);
                cx.notify();
            })
        };
        let scene = choice("scene", "Cards", &["10k", "100k"], cx, |lab, index| lab.set_scene_size(SCENE_SIZES[index]));
        let grid = choice("grid", "Grid", &["Lines", "Dots"], cx, |lab, index| {
            lab.grid_style = [GridStyle::Lines, GridStyle::Dots][index];
        });
        let map = choice("map", "Map", &["Whole scene", "Current view", "Hidden"], cx, |lab, index| {
            lab.show_minimap = index < 2;
            if index < 2 {
                lab.whole_scene_map = index == 0;
            }
        });
        let renderer = choice("renderer", "Renderer", &["Auto", "CPU"], cx, |lab, index| {
            if (index == 1) != lab.fractal.cpu_forced() {
                lab.fractal.toggle_backend();
            }
        });
        let preview = radix::Switch::new("canvas-preview-switch")
            .look(look)
            .label("Preview")
            .checked(false)
            .on_value_changed(cx, |lab: &mut Lab, on, cx| {
                lab.cancel_interaction();
                lab.show_minimap = on;
                cx.notify();
            })
            .spawn(cx);
        Self { scene, grid, map, renderer, preview, reset, overview, run, cancel, reveal }
    }
}

impl Lab {
    pub(super) fn set_scene_size(&mut self, count: usize) {
        if self.scene.len() != count {
            self.scene = make_scene(count);
            self.selected.clear();
            self.invalidate_preview();
        }
    }

    pub(super) fn overview_zoom(&mut self) {
        self.camera = Camera2D::default();
        let viewport = self.viewport();
        self.camera.zoom_at(WorldPoint::new(viewport.x / 2., viewport.y / 2.), viewport, 0.01);
    }

    fn run_sweep(&mut self) {
        if self.fractal.enabled {
            self.status = Some("Select Spatial Canvas to run its camera sweep.".into());
        } else if self.profile.is_none() {
            match tempfile::Builder::new().prefix("xgpui-canvas-sweep-").tempdir() {
                Ok(folder) => {
                    self.profile = Some(profile::Profile::new(folder.path().join("report.json")));
                    self.results = Some(folder.keep());
                    self.status = Some("Running 450 frames; keep this pane visible and in the foreground.".into());
                }
                Err(error) => self.status = Some(error.to_string()),
            }
        }
    }

    /// Keep the controls in step with state changed by keys, the sweep or the window.
    fn sync_controls(&self, cx: &mut Context<Self>) {
        let controls = &self.controls;
        let scene = SCENE_SIZES.iter().position(|&count| count == self.scene.len()).unwrap_or(0);
        controls.scene.sync(labels(&["10k", "100k"]), scene, cx);
        controls
            .grid
            .sync(labels(&["Lines", "Dots"]), usize::from(matches!(self.grid_style, GridStyle::Dots)), cx);
        let map = match (self.show_minimap, self.whole_scene_map) {
            (false, _) => 2,
            (true, whole) => usize::from(!whole),
        };
        controls.map.sync(labels(&["Whole scene", "Current view", "Hidden"]), map, cx);
        controls.renderer.sync(labels(&["Auto", "CPU"]), usize::from(self.fractal.cpu_forced()), cx);
        if *controls.preview.read(cx).data() != self.show_minimap {
            let on = self.show_minimap;
            controls.preview.update(cx, |switch, cx| switch.set_data(on, cx));
        }
    }

    /// The page around the viewport: Spatial Canvas, or Mandelbrot when the fractal is on.
    pub(super) fn screen(&self, viewport: AnyElement, drawn: Drawn, wide: bool, cx: &mut Context<Self>) -> LabScreen {
        self.sync_controls(cx);
        let look = &self.look;
        let choice = |choice: &Choice| labeled_control(look, choice.clone().into_element());
        let hero = Hero::full(viewport);
        if self.fractal.enabled {
            return LabScreen::new("mandelbrot", look.clone())
                .crumb("xgpui-canvas", "Mandelbrot")
                .title("Mandelbrot")
                .lede("Explore the Mandelbrot set from the whole set down to 10¹⁰× zoom, rendered on the GPU or in f64 on the CPU.")
                .hero(hero)
                .caption(self.fractal_caption())
                .control(choice(&self.controls.renderer))
                .control(self.controls.preview.clone())
                .action(self.controls.reset.clone())
                .when_some(self.results.as_ref(), |screen, _| screen.action(self.controls.reveal.clone()))
                .notice(self.status.clone())
                .notice(self.fractal.error().map(|error| format!("Frame error: {error}")))
                .cards(self.fractal_cards(wide))
                .note(FRACTAL_NOTE)
                .credits(FRACTAL_CREDITS);
        }
        let sweep = if self.profile.is_some() {
            &self.controls.cancel
        } else {
            &self.controls.run
        };
        LabScreen::new("spatial-canvas", look.clone())
            .crumb("xgpui-canvas", "Spatial Canvas")
            .title("Spatial Canvas")
            .lede("Explore a scene of 10,000 or 100,000 cards on an infinite canvas: pan, zoom, select and snap, with level of detail as cards shrink.")
            .hero(hero)
            .caption(self.canvas_caption(drawn))
            .control(choice(&self.controls.scene))
            .control(choice(&self.controls.grid))
            .control(choice(&self.controls.map))
            .action(self.controls.overview.clone())
            .action(self.controls.reset.clone())
            .action(sweep.clone())
            .when_some(self.results.as_ref(), |screen, _| screen.action(self.controls.reveal.clone()))
            .notice(self.status.clone())
            .cards(self.canvas_cards(drawn))
            .note(CANVAS_NOTE)
            .credits(CANVAS_CREDITS)
    }

    fn canvas_caption(&self, drawn: Drawn) -> String {
        format!(
            "{:.1}% zoom · {} of {} cards visible · {} drawn in full · {} proxies ({} points) · {} selected",
            self.camera.zoom() * 100.,
            grouped(drawn.cards + drawn.proxies),
            grouped(self.scene.len()),
            grouped(drawn.cards),
            grouped(drawn.proxies),
            grouped(drawn.points),
            grouped(self.selected.len()),
        )
    }

    fn canvas_cards(&self, drawn: Drawn) -> [LabCard; 4] {
        let center = self.center();
        let sweep = self.profile.as_ref().map(|profile| profile.frame.min(SWEEP_FRAMES));
        let (state, active) = match (sweep, &self.results) {
            (Some(_), _) => ("Running", true),
            (None, Some(_)) => ("Saved", false),
            (None, None) => ("Idle", false),
        };
        [
            LabCard::new("Scene")
                .icon(Icon::LayoutGrid)
                .subtitle("160 × 80 cards, 100 per row")
                .values([("cards", grouped(self.scene.len())), ("selected", grouped(self.selected.len()))]),
            LabCard::new("Level of detail").icon(Icon::Layers).subtitle("Full cards, proxies, points").values([
                ("full", grouped(drawn.cards)),
                ("proxy", grouped(drawn.proxies - drawn.points)),
                ("point", grouped(drawn.points)),
            ]),
            LabCard::new("Camera").icon(Icon::Scan).subtitle("Zoom 1% to 6400%").values([
                ("zoom", format!("{:.1}%", self.camera.zoom() * 100.)),
                ("x", format!("{:.0}", center.x)),
                ("y", format!("{:.0}", center.y)),
            ]),
            LabCard::new("Sweep")
                .icon(Icon::Gauge)
                .subtitle("450-frame camera sweep")
                .badge(self.badge(state, active))
                .values([("frame", sweep.map_or("—".into(), |frame| format!("{frame} / {SWEEP_FRAMES}")))]),
        ]
    }

    fn fractal_caption(&self) -> String {
        let mut parts =
            vec![format!("{} zoom", zoom_factor(self.camera.zoom())), format!("c = {}", self.complex_center())];
        parts.extend(self.fractal.limit().map(|limit| format!("{limit} iterations")));
        parts.extend(self.fractal.route().map(str::to_owned));
        parts.extend(self.fractal.frame_size().map(|[w, h]| format!("{w}×{h}")));
        parts.push(if self.fractal.refined { "refined" } else { "interactive" }.into());
        parts.join(" · ")
    }

    fn fractal_cards(&self, wide: bool) -> [LabCard; 4] {
        let route = self.fractal.route();
        let gpu = route == Some("Metal GPU");
        let refined = self.fractal.refined;
        [
            LabCard::new("Renderer")
                .icon(Icon::Cpu)
                .subtitle(format!("GPU up to {}×, then f64 CPU", mandelbrot::GPU_MAX_ZOOM))
                .badge(self.badge(if self.fractal.cpu_forced() { "CPU" } else { "Auto" }, gpu))
                .values([
                    ("route", route.unwrap_or("—").to_owned()),
                    ("frame", self.fractal.frame_size().map_or("—".into(), |[w, h]| format!("{w}×{h}"))),
                ]),
            LabCard::new("Iterations").icon(Icon::Repeat).subtitle("Limit grows with zoom").values([
                ("limit", self.fractal.limit().map_or("—".into(), |limit| limit.to_string())),
                ("zoom", zoom_factor(self.camera.zoom())),
            ]),
            LabCard::new("Frame")
                .icon(Icon::Frame)
                .subtitle("Whole frames, refined when settled")
                .badge(self.badge(if refined { "Refined" } else { "Interactive" }, refined))
                .values([
                    ("settle", format!("{} ms", mandelbrot::SETTLE.as_millis())),
                    ("blend", format!("{} ms", mandelbrot::SHARPEN_BLEND.as_millis())),
                ]),
            LabCard::new("Color")
                .icon(Icon::Palette)
                .subtitle(if wide {
                    "Display P3 wide SDR"
                } else {
                    "Mapped sRGB fallback"
                })
                .badge(self.badge(if wide { "Wide" } else { "Mapped" }, wide))
                .values([("space", "linear P3"), ("output", if wide { "P3" } else { "sRGB" })]),
        ]
    }

    /// World point at the center of the viewport.
    fn center(&self) -> WorldPoint {
        let viewport = self.viewport();
        self.camera.screen_to_world(WorldPoint::new(viewport.x / 2., viewport.y / 2.), viewport)
    }

    /// The viewport center as a point of the complex plane (see `render_cpu`'s mapping).
    fn complex_center(&self) -> String {
        let center = self.center();
        let (re, im) = (center.x / 300. - 0.5, center.y / 300.);
        // Enough decimals to tell neighboring pixels apart at this zoom.
        let digits = ((self.camera.zoom() * 300.).log10().ceil().max(0.) as usize + 1).clamp(4, 14);
        let sign = if im < 0. { '−' } else { '+' };
        format!("{} {sign} {:.digits$}i", signed(re, digits), im.abs())
    }

    /// `Soft` for an active or positive state, `Outline` otherwise.
    fn badge(&self, label: &'static str, active: bool) -> Badge {
        Badge::new(&self.look, label).variant(if active {
            BadgeVariant::Soft
        } else {
            BadgeVariant::Outline
        })
    }
}

/// A count with thousands separators: `10,000`.
fn grouped(count: usize) -> String {
    let digits = count.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// A zoom factor: `1.00×`, `64.0×`, `3.2e7×`.
fn zoom_factor(zoom: f64) -> String {
    match zoom {
        z if z < 10. => format!("{z:.2}×"),
        z if z < 10_000. => format!("{z:.1}×"),
        z => format!("{z:.1e}×"),
    }
}

/// A number with a typographic minus sign.
fn signed(value: f64, digits: usize) -> String {
    let text = format!("{:.digits$}", value.abs());
    if value < 0. && text.chars().any(|c| c.is_ascii_digit() && c != '0') {
        format!("−{text}")
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::{grouped, signed, zoom_factor};

    #[::core::prelude::v1::test]
    fn values_format_for_reading() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(10_000), "10,000");
        assert_eq!(grouped(100_000), "100,000");
        assert_eq!(zoom_factor(1.), "1.00×");
        assert_eq!(zoom_factor(64.), "64.0×");
        assert_eq!(zoom_factor(3.2e7), "3.2e7×");
        assert_eq!(signed(-0.5, 4), "−0.5000");
        assert_eq!(signed(-0.00001, 4), "0.0000");
    }
}

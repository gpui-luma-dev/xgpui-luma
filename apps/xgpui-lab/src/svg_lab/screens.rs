//! Shared presentation for the SVG probes; scheduler, paths and camera stay with SvgPage.
use super::*;
use crate::theme::ColorVizLookExt;
use gpui::px;
use gpui::{prelude::*, AnyElement, div};
use crate::screen::{Hero, LabScreen, PanelShell, ParametersPanel, properties, labeled_control};
impl SvgPage {
    pub(super) fn switch_value(&self, action: Action) -> bool {
        match action {
            Action::Clip => self.clipped,
            Action::Shadow => self.shadow,
            Action::Animate => self.animate,
            Action::Eager => self.eager,
            _ => false,
        }
    }
    pub(super) fn screen(
        &self,
        actions: &[Action],
        parameters: &[Parameter],
        result: anyhow::Result<AnyElement>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.active_tab == "Benchmarks" {
            return div()
                .id("svg-lab-scroll")
                .size_full()
                .overflow_y_scroll()
                .p_4()
                .flex()
                .flex_col()
                .gap_4()
                .text_color(self.look.chrome().body_text)
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_2()
                        .children(
                            self.buttons
                                .iter()
                                .filter(|(action, _)| {
                                    actions.contains(action)
                                        && !(matches!(action, Action::Clip | Action::Shadow)
                                            && self.asset.as_ref().is_ok_and(|asset| asset.source.is_some()))
                                })
                                .map(|(action, button)| {
                                    let id = format!("svg-action-{action:?}");
                                    div().id(id.clone()).debug_selector(move || id.clone()).child(button.clone())
                                }),
                        )
                        .children(
                            self.choices
                                .iter()
                                .filter(|(action, _)| actions.contains(action))
                                .map(|(_, choice)| choice.clone()),
                        ),
                )
                .child(div().flex().flex_wrap().gap_4().children(
                    self.sliders.iter().filter(|(parameter, _)| parameters.contains(parameter)).map(
                        |(parameter, slider)| {
                            div()
                                .w(px(190.))
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(format!("{}: {:.2}", parameter.config().0, self.values[*parameter as usize]))
                                .child(slider.clone())
                        },
                    ),
                ))
                .when(self.active_tab != "Rendering", |view| {
                    view.child(format!(
                        "Output budget: {:.2} MiB · Zoom {:.2}× · Display scale {:.2}×",
                        self.policy.max_output_bytes as f64 / MIB as f64,
                        self.values[0],
                        self.density
                    ))
                })
                .when(self.loading, |view| view.child("Loading artwork in the background…"))
                .children(
                    self.load_error
                        .as_ref()
                        .map(|error| div().child(format!("Load failed; previous artwork retained: {error}"))),
                )
                .child(match result {
                    Ok(view) => view,
                    Err(error) => div().child(format!("SVG lab unavailable: {error:#}")).into_any_element(),
                })
                .into_any_element();
        }
        let (id, title, lede, note) = match self.active_tab.as_ref() {
            "Scaling" => (
                "svg-scaling",
                "Scaling",
                "Compare a magnified texture with an SVG that redraws as you zoom.",
                "Drag either view to pan both. Wheel or pinch zooms around the pointer; presets zoom around the panel center. Both views share the artwork and camera.",
            ),
            "Path Effects" => (
                "svg-paths",
                "Path Effects",
                "Explore how trimming and dashing change the same measured paths.",
                "Trim uses total length across contours; dash phase restarts per contour. Editing a numeric parameter pauses motion. Measurement is retained before native stroke tessellation.",
            ),
            "Scheduling & Cache" => (
                "svg-cache",
                "Scheduling & Cache",
                "Explore how a gallery shares a limited pool of raster jobs and image memory.",
                "Scroll to explore; Command-wheel or pinch zooms. Open a folder for real artwork. Outputs remain independent even when documents are shared.",
            ),
            _ => (
                "svg-rendering",
                "Rendering",
                "Compare the same artwork as a fixed image, an SVG and native paths.",
                "Rendering stays at 1× artwork size. Native paths approximate the built-in gradients and shadow; this compares capabilities rather than pixel equivalence. Scaling has its own retained camera.",
            ),
        };
        for (number, value) in self.numbers.iter().zip(self.values) {
            number.observe(value, cx);
        }
        let caption = if title == "Path Effects" {
            format!(
                "Trim {:.0}% · dash {} · phase {:.1} · width {:.0} px · {}",
                self.effective_trim() * 100.,
                self.dash,
                self.effective_phase(),
                self.values[5],
                if self.animate && self.motion_allowed {
                    "Animating"
                } else {
                    "Paused"
                }
            )
        } else if title == "Scheduling & Cache" {
            format!(
                "{} tiles · {} documents · {} · {:.2}× zoom",
                self.tiles,
                self.gallery_docs.len().max(1),
                if self.eager { "eager" } else { "visible + overscan" },
                self.values[0]
            )
        } else {
            format!(
                "{} · {:.2}× zoom · {:.2}× display density",
                self.source_label(),
                if title == "Rendering" { 1. } else { self.camera.zoom },
                self.density
            )
        };
        let mut page = LabScreen::new(id,self.look.clone()).crumb("xgpui-svg",title).title(title).lede(lede).caption(caption).note(note)
            .credits("Raster budgets account for output image reservations, excluding SVG filter intermediates, total process RSS and driver VRAM. CPU parse/raster observations are not GPU or presentation timing.");
        match result {
            Ok(view) => page = page.hero(Hero::full(view)),
            Err(e) => page = page.notice(Some(format!("SVG lab unavailable: {e:#}"))),
        }
        for (action, choice) in &self.choices {
            if actions.contains(action) {
                let control = if *action == Action::Background {
                    // Reserve enough space for Transparent so selection never reflows the row.
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(choice.label())
                        .child(div().w(px(160.)).flex_shrink_0().child(choice.selector.clone()))
                } else {
                    choice.clone().into_element()
                };
                page = page.control(labeled_control(&self.look, control));
            }
        }
        for (action, switch) in &self.switches {
            if !actions.contains(action)
                || (matches!(action, Action::Clip | Action::Shadow)
                    && self.asset.as_ref().is_ok_and(|a| a.source.is_some()))
            {
                continue;
            }
            let on = self.switch_value(*action);
            if *switch.read(cx).data() != on {
                switch.update(cx, |s, cx| s.set_data(on, cx));
            }
            let selector = format!("svg-action-{action:?}");
            page =
                page.control(div().id(selector.clone()).debug_selector(move || selector.clone()).child(switch.clone()));
        }
        let mut presets = div().flex().flex_wrap().gap_2();
        for (action, button) in &self.buttons {
            if !actions.contains(action) || self.switches.iter().any(|(a, _)| a == action) {
                continue;
            }
            let selector = format!("svg-action-{action:?}");
            let button = div().id(selector.clone()).debug_selector(move || selector.clone()).child(button.clone());
            if matches!(
                action,
                Action::Zoom1 | Action::Zoom2 | Action::Zoom4 | Action::Zoom8 | Action::Zoom16 | Action::DevicePixels
            ) {
                presets = presets.child(button);
            } else {
                page = page.action(button);
            }
        }
        let diagnostics = self.diagnostics();
        if parameters.is_empty() {
            page = page.section_panel(diagnostics);
        } else {
            page = page.panels(
                ParametersPanel::new(if title == "Scaling" { "Camera" } else { "Parameters" })
                    .params(parameters.iter().map(|p| self.numbers[*p as usize].param()))
                    .footer(presets),
                diagnostics,
            );
        }
        if title == "Scaling" {
            page = page.equal_panel_widths();
        }
        if self.loading {
            page = page.notice(Some("Loading artwork in the background…"));
        }
        if let Some(error) = &self.load_error {
            page = page.notice(Some(format!("Load failed; previous artwork retained: {error}")));
        }
        page.into_any_element()
    }
    pub(super) fn diagnostics(&self) -> PanelShell {
        let mut rows = vec![("Output budget", format!("{:.2} MiB", self.policy.max_output_bytes as f64 / MIB as f64))];
        if self.active_tab == "Scheduling & Cache"
            && let Ok(scheduler) = &self.scheduler
        {
            let s = scheduler.stats();
            let l = scheduler.limits();
            rows.extend([
                ("Job limit", l.max_jobs.to_string()),
                ("Pool", format!("{} MiB", l.max_bytes / MIB)),
                ("Members / active / queued", format!("{} / {} / {}", s.members, s.active_jobs, s.queued_jobs)),
                (
                    "Retained / in-flight",
                    format!(
                        "{:.2} / {:.2} MiB",
                        s.retained_bytes as f64 / MIB as f64,
                        s.in_flight_bytes as f64 / MIB as f64
                    ),
                ),
                ("Peak managed", format!("{:.2} MiB", s.peak_managed_bytes as f64 / MIB as f64)),
            ]);
        }
        if self.active_tab == "Scaling"
            && let Ok(asset) = &self.asset
        {
            if let Ok(r) = self.policy.resolution(asset.logical, self.camera.zoom, self.density) {
                let p = self.policy.preview(r.output);
                rows.extend([
                    ("Requested", format!("{}×{} px", i32::from(r.requested.width), i32::from(r.requested.height))),
                    (
                        "Output",
                        format!(
                            "{}×{} px · {:.2} MiB{}",
                            i32::from(r.output.width),
                            i32::from(r.output.height),
                            xgpui_svg::image_bytes(r.output) as f64 / MIB as f64,
                            if r.limited() { " · capped" } else { "" }
                        ),
                    ),
                    (
                        "Preview",
                        format!(
                            "{}×{} px · {} px edge",
                            i32::from(p.width),
                            i32::from(p.height),
                            self.policy.preview_edge
                        ),
                    ),
                    ("Settle", format!("{} ms", self.policy.settle_delay.as_millis())),
                ]);
            }
            for density in [1., 2., 3.] {
                if let Ok(r) = self.policy.resolution(asset.logical, self.camera.zoom, density) {
                    rows.push((
                        match density as u8 {
                            1 => "1× density estimate",
                            2 => "2× density estimate",
                            _ => "3× density estimate",
                        },
                        format!(
                            "{}×{} → {}×{} px · {:.2} MiB{}",
                            i32::from(r.requested.width),
                            i32::from(r.requested.height),
                            i32::from(r.output.width),
                            i32::from(r.output.height),
                            xgpui_svg::image_bytes(r.output) as f64 / MIB as f64,
                            if r.limited() { " · capped" } else { "" }
                        ),
                    ));
                }
            }
        }
        if self.active_tab == "Rendering"
            && let Ok(a) = &self.asset
        {
            rows.extend([
                ("Parse", format!("{:.3} ms", a.parse_ms)),
                ("Baseline raster", format!("{:.3} ms", a.raster_ms)),
                ("Clip / shadow", format!("{} / {}", self.clipped, self.shadow)),
            ]);
        }
        let status = if self.active_tab == "Rendering" {
            &self.rendering_status
        } else {
            &self.status
        };
        if matches!(self.active_tab.as_ref(), "Rendering" | "Scaling") {
            // Rebuilding artwork clears status until the element reports again.
            // Keep the same rows mounted so the panel and footer do not jump.
            rows.extend([
                ("Phase", status.as_ref().map_or_else(|| "Pending".into(), |s| format!("{:?}", s.phase))),
                (
                    "Displayed",
                    status.as_ref().and_then(|s| s.displayed).map_or_else(
                        || "Pending".into(),
                        |size| format!("{}×{} px", i32::from(size.width), i32::from(size.height)),
                    ),
                ),
                (
                    "Raster",
                    status
                        .as_ref()
                        .and_then(|s| s.raster_ms)
                        .map_or_else(|| "Pending".into(), |ms| format!("{ms:.3} ms")),
                ),
                ("Completed", status.as_ref().map_or(0, |s| s.completed_rasters).to_string()),
            ]);
            if let Some(error) = status.as_ref().and_then(|s| s.error.as_ref()) {
                rows.push(("Raster error", error.clone()));
            }
        }
        PanelShell::new("Rendering / live").child(properties(&self.look, rows))
    }
}

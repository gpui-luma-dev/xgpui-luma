//! Linear-light image adjustments and explicit sRGB / Display P3 presentation.
use gpui::{Context, IntoElement, Window, prelude::*, px};
use xgpui_media::pipeline::ImagePrimaries;

use super::{Action, MediaPage, Parameter};
use crate::screen::{ChainPanel, ComparePane, Hero, LabScreen, Param, ParametersPanel, labeled_control};

pub(super) const VIEW_PARAMETERS: [Parameter; 5] =
    [Parameter::Zoom, Parameter::PanX, Parameter::PanY, Parameter::Opacity, Parameter::Radius];
pub(super) const LINEAR_PARAMETERS: [Parameter; 4] =
    [Parameter::Exposure, Parameter::LinearContrast, Parameter::LinearSaturation, Parameter::Vignette];

impl MediaPage {
    pub(super) fn render_adjustments(&mut self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_switches(cx);
        let float = self.active_tab == "Float / P3";
        let (id, title, lede) = if float {
            (
                "media-float",
                "Float / P3",
                "Compare a colorful image in a standard preview and in the colors your display can show.",
            )
        } else {
            (
                "media-linear",
                "Linear Adjustments",
                "Compare an original image with changes to its brightness, contrast, saturation and edges.",
            )
        };
        let measured = f32::from(self.bounds[0].size.width);
        let width = if measured > 1. { measured } else { 560. };
        let height = width * 2. / 3.;
        let (hero, caption, notice) = if float {
            self.float_hero(window, width, height, cx)
        } else {
            let scale = self.display_scale(width, height, cx).unwrap_or(0.);
            let name = self
                .fixtures
                .as_ref()
                .map_or("Generating test images…", |fixtures| fixtures.samples[self.selected].name.as_str());
            (
                Hero::compare(
                    ComparePane::new("Original", self.compare_view(false, width, height, true, cx))
                        .subtitle("Retained media source"),
                    ComparePane::new("Adjusted", self.compare_view(true, width, height, true, cx))
                        .subtitle("Linear sRGB"),
                ),
                format!("{name} · shown at {scale:.2}× · exposure {:+.2} EV", self.exposure()),
                None,
            )
        };
        let mut screen = LabScreen::new(id, self.look.clone())
            .crumb("xgpui-media", title)
            .title(title)
            .lede(lede)
            .hero(hero)
            .caption(caption)
            .notice(notice)
            .notice(self.error.as_ref().map(|error| format!("Media error: {error}")));
        if !float {
            screen = self.screen_choices(screen, &[Action::Sample]);
            screen = self.screen_actions(screen, &[Action::Open, Action::Reload]);
        }
        screen = self.screen_switches(
            screen,
            if float {
                &[Action::Animate, Action::Wide]
            } else {
                &[Action::Animate]
            },
        );
        screen = self.screen_actions(screen, &[Action::Reset]);
        let mut parameters = self.parameter_panel(
            "Image parameters",
            &VIEW_PARAMETERS.into_iter().chain(LINEAR_PARAMETERS).collect::<Vec<_>>(),
        );
        if let Some(reset) = self.button(Action::Filters) {
            parameters = parameters.action(reset);
        }
        let screen = screen.panels(parameters, self.adjustment_chain())
            .note(if float { "Both panes reuse one RGBA16F source. Wide output appears only when the window negotiates Display P3; otherwise both panes show an explicit sRGB preview." } else { "Both panes share the same camera and source upload. Only the right pane applies the linear-light adjustments; opacity and rounded corners affect both." })
            .credits(if float { "Generated linear Display P3 pixels, 1280×800. This compares presentation paths, not HDR output. Changes to the sampling transform do not regenerate the source." } else { "Synthetic test images generated offline by the lab. Exposure, contrast and saturation compose in linear sRGB; vignette darkens the edges afterwards." });
        self.camera_screen(screen, cx)
    }

    fn float_hero(
        &self,
        window: &Window,
        width: f32,
        height: f32,
        cx: &mut Context<Self>,
    ) -> (Hero, String, Option<String>) {
        let wide = self.wide && window.color_status().effective() == gpui::WindowColorMode::WideGamutSdr;
        let mut notice = None;
        let mut native = false;
        let [left, right] = [false, true].map(|right| {
            let mut viewport = self.camera_viewport(usize::from(right), width, height, true, cx);
            if let Some(image) = &self.float_image {
                match self.adjustments(ImagePrimaries::DisplayP3) {
                    Ok(adjustments) => {
                        let art_w = width * self.values[0];
                        let art_h = art_w * image.height() as f32 / image.width() as f32;
                        let make_view = || {
                            xgpui_media::image_view(image.clone())
                                .adjustments(adjustments)
                                .absolute()
                                .left(px((width - art_w) * 0.5 + self.values[1]))
                                .top(px((height - art_h) * 0.5 + self.values[2]))
                                .w(px(art_w))
                                .h(px(art_h))
                                .rounded(px(self.values[4]))
                                .opacity(self.values[3])
                        };
                        let view = if right && wide {
                            match make_view().wide_gamut() {
                                Ok(view) => {
                                    native = true;
                                    view
                                }
                                Err(error) => {
                                    notice = Some(format!("Wide output unavailable; showing sRGB: {error}"));
                                    make_view()
                                }
                            }
                        } else {
                            make_view()
                        };
                        viewport = viewport.child(view);
                    }
                    Err(error) => notice = Some(format!("Adjustments unavailable: {error}")),
                }
            }
            viewport
        });
        let output = if native {
            "native linear Display P3"
        } else {
            "sRGB preview"
        };
        let source = self.float_image.as_ref().map_or_else(
            || {
                self.float_job
                    .as_ref()
                    .map_or("Waiting for float source".into(), |job| format!("{:?}", job.status()))
            },
            |image| {
                format!(
                    "{}×{} · {:?} · {} mip levels · {} uploads · {:?}",
                    image.width(),
                    image.height(),
                    image.format(),
                    image.mip_count(),
                    image.upload_count(),
                    image.upload_status()
                )
            },
        );
        (
            Hero::compare(
                ComparePane::new("sRGB preview", left),
                ComparePane::new(if native { "Native Display P3" } else { "sRGB preview" }, right),
            ),
            format!("{source} · {output} · exposure {:+.2} EV", self.exposure()),
            notice,
        )
    }

    pub(super) fn parameter_panel(&self, title: &'static str, parameters: &[Parameter]) -> ParametersPanel {
        ParametersPanel::new(title).params(parameters.iter().filter_map(|parameter| {
            let field = self.filter_fields.iter().find(|(id, _)| id == parameter)?.1.clone();
            let slider = self.sliders.iter().find(|(id, _)| id == parameter)?.1.clone();
            Some(Param::new(parameter.config().0, field, slider).range(parameter.range_label()))
        }))
    }

    pub(super) fn adjustment_chain(&self) -> ChainPanel {
        ChainPanel::new("Linear-light chain")
            .step("Exposure", format!("{:+.2} EV", self.exposure()), self.exposure().abs() > f64::EPSILON)
            .step("Contrast", format!("× {:.2} around 0.18", self.values[10]), self.values[10] != 1.)
            .step("Saturation", format!("× {:.2}", self.values[11]), self.values[11] != 1.)
            .step("Vignette", format!("{:.2}", self.values[12]), self.values[12] > 0.)
            .note("Animation adds a changing exposure offset. Identity values leave the image unchanged.")
    }

    pub(super) fn screen_choices(&self, mut screen: LabScreen, actions: &[Action]) -> LabScreen {
        for action in actions {
            if let Some(choice) = self.choice(*action) {
                screen = screen.control(labeled_control(&self.look, choice.into_element()));
            }
        }
        screen
    }

    pub(super) fn screen_actions(&self, screen: LabScreen, actions: &[Action]) -> LabScreen {
        actions.iter().filter_map(|action| self.button(*action)).fold(screen, LabScreen::action)
    }

    pub(super) fn screen_switches(&self, screen: LabScreen, actions: &[Action]) -> LabScreen {
        self.state_switches
            .iter()
            .filter(|(action, _)| actions.contains(action))
            .fold(screen, |screen, (_, switch)| screen.control(switch.clone()))
    }
}

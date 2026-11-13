//! Retained image inspection and atlas/mipmap comparisons in the standard lab layout.
use gpui::{Context, IntoElement, Window};

use super::{Action, MIB, MediaPage};
use crate::screen::{ComparePane, Hero, LabCard, LabScreen, PanelShell, labeled_control, properties};

impl MediaPage {
    pub(super) fn render_image_screen(&mut self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mipmaps = self.active_tab == "Textures & Mipmaps";
        let (id, title, lede) = if mipmaps {
            (
                "media-mipmaps",
                "Textures & Mipmaps",
                "Compare how two ways of drawing an image preserve fine detail as you zoom out.",
            )
        } else {
            (
                "media-images",
                "Images",
                "Explore an image up close, move around it, and inspect its retained source.",
            )
        };
        self.sync_switches(cx);
        // Images uses viewport 1; the comparison's left pane uses viewport 0.
        let measured = f32::from(self.bounds[usize::from(!mipmaps)].size.width);
        let width = if measured > 1. { measured } else { 560. };
        let height = width * 2. / 3.;
        let scale = self.display_scale(width, height, cx);
        let hero = if mipmaps {
            self.comparison_hero(width, height, scale, cx)
        } else {
            Hero::stage(ComparePane::new("Retained media image", self.compare_view(true, width, height, true, cx)))
        };
        let sample = self.fixtures.as_ref().map(|fixtures| &fixtures.samples[self.selected]);
        let caption = sample.map_or_else(
            || "Generating test images…".into(),
            |sample| {
                format!("{} · {}×{} · shown at {:.2}×", sample.name, sample.width, sample.height, scale.unwrap_or(0.))
            },
        );
        let mut screen = LabScreen::new(id, self.look.clone())
            .crumb("xgpui-media", title)
            .title(title)
            .lede(lede)
            .hero(hero)
            .caption(caption);
        if let Some(choice) = self.choice(Action::Sample) {
            screen = screen.control(labeled_control(&self.look, choice.into_element()));
        }
        if mipmaps {
            screen = screen.control(self.sweep_switch.clone());
        }
        for action in [Action::Open, Action::Reload, Action::Reset] {
            if let Some(button) = self.button(action) {
                screen = screen.action(button);
            }
        }
        let image = self.image.as_ref();
        let cards = [
            LabCard::new("Source")
                .subtitle(sample.map_or("Generating…".into(), |sample| sample.name.clone()))
                .values([("size", sample.map_or("—".into(), |sample| format!("{}×{}", sample.width, sample.height)))]),
            LabCard::new("Upload")
                .subtitle(image.map_or_else(
                    || self.job.as_ref().map_or("Waiting".into(), |job| format!("{:?}", job.status())),
                    |image| format!("{:?}", image.upload_status()),
                ))
                .values([
                    ("uploads", image.map_or("—".into(), |image| image.upload_count().to_string())),
                    (
                        "source",
                        image.map_or("—".into(), |image| {
                            format!("{:.2} MiB", image.payload_bytes() as f64 / MIB as f64)
                        }),
                    ),
                ]),
        ];
        let mut details =
            vec![("Source", sample.map_or("Generating…".into(), |sample| sample.description.to_string()))];
        if mipmaps {
            let ratio = scale.unwrap_or(0.) * window.scale_factor();
            let levels = image.map_or(0, |image| image.mip_count());
            let lod = if ratio > 0. {
                (-ratio.log2()).max(0.).min(levels.saturating_sub(1) as f32)
            } else {
                0.
            };
            details.push(("Footprint", format!("{ratio:.4} device pixels / source pixel")));
            details.push(("Estimated LOD", format!("{lod:.2}")));
            if let Some(image) = image {
                for level in 0..levels {
                    details.push((
                        "Mip",
                        format!("{level}: {}×{}", (image.width() >> level).max(1), (image.height() >> level).max(1)),
                    ));
                }
            }
        }
        let screen = screen
            .notice(self.error.as_ref().map(|error| format!("Media error: {error}")))
            .cards(cards)
            .panels(
                self.parameter_panel("View parameters", &super::adjustment_screens::VIEW_PARAMETERS).subtitle("Wheel or pinch to zoom; drag to pan."),
                PanelShell::new(if mipmaps { "Sampling probe" } else { "Image details" }).child(properties(&self.look, details)),
            )
            .note(if mipmaps { "Both panes share the same camera. Sweep zoom crosses mip transitions; the LOD readout estimates the footprint and does not force the GPU's mip selection." } else { "Open a local image or choose a generated sample. Panning and zooming reuse the decoded source; Reset view restores the camera, opacity and corners." })
            .credits("Built-in samples are synthetic images generated offline by the lab. Source payload excludes the additional mip levels. Ready reports host submission, not GPU completion.");
        self.camera_screen(screen, cx)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;

    #[test]
    fn image_screens_keep_slots_camera_and_reset_controls() {
        use xgpui_luma::controls::button::ButtonEvent;
        let mut app = gpui::TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| MediaPage::new(crate::theme::default_look(), cx));
        page.update(cx, |page, _| {
            page.fixtures = Some(super::super::fixtures::Fixtures::new(None).unwrap());
        });
        for (tab, slots) in [
            (
                "Images",
                [
                    "media-images-crumb",
                    "media-images-header",
                    "media-images-hero",
                    "media-images-caption",
                    "media-images-controls",
                    "media-images-cards",
                    "media-images-panels",
                ],
            ),
            (
                "Textures & Mipmaps",
                [
                    "media-mipmaps-crumb",
                    "media-mipmaps-header",
                    "media-mipmaps-hero",
                    "media-mipmaps-caption",
                    "media-mipmaps-controls",
                    "media-mipmaps-cards",
                    "media-mipmaps-panels",
                ],
            ),
        ] {
            page.update(cx, |page, cx| page.activate(tab.into(), cx));
            cx.run_until_parked();
            for slot in slots {
                assert!(cx.debug_bounds(slot).is_some(), "{tab}: {slot}");
            }
            page.update(cx, |page, cx| {
                page.zoom_at(gpui::point(gpui::px(20.), gpui::px(10.)), 1.5, cx);
                assert_eq!(page.values[0], 1.5);
                assert_ne!(page.values[1], 0.);
            });
            let reset = cx.update(|_, cx| {
                page.read(cx).buttons.iter().find(|(action, _)| *action == Action::Reset).unwrap().1.clone()
            });
            reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
            cx.run_until_parked();
            cx.update(|_, cx| {
                assert_eq!(&page.read(cx).values[..3], &[1., 0., 0.]);
            });
        }
    }
}

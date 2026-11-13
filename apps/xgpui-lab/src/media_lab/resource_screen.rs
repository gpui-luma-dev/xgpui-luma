//! LabScreen presentation of the original media-lab upload stress test.
use gpui::{Context, IntoElement, div, prelude::*, px};
use xgpui_luma_look_radix::{self as radix, BadgeVariant};
use xgpui_media::{MediaLimits, UploadLimits};

use super::{Action, MIB, MediaPage, resource_model::Phase};
use crate::screen::{ComparePane, Hero, LabCard, LabScreen, PanelShell, TextRole, card_surface, properties, text};

impl MediaPage {
    pub(super) fn render_resources(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let lab = &self.resources;
        // As in the original stress() panel, every request keeps a numbered chip
        // while only snapshots still holding a source show an image preview.
        let chips = div().flex().flex_wrap().gap_1().children(lab.burst.iter().enumerate().map(|(index, slot)| {
            card_surface(div(), &self.look)
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .w(px(68.))
                .min_h(px(60.))
                .p_1()
                .gap_1()
                .debug_selector(move || format!("media-resource-result-{index}"))
                .child(text(&self.look, TextRole::Value, format!("{:02}", index + 1)))
                .child(radix::Badge::new(&self.look, slot.phase.label()).variant(
                    if matches!(slot.phase, Phase::Ready | Phase::Retired) {
                        BadgeVariant::Soft
                    } else {
                        BadgeVariant::Outline
                    },
                ))
        }));
        let previews = div().flex().flex_wrap().gap_3().min_h(px(72.)).children(
            lab.burst.iter().enumerate().filter_map(|(index, slot)| {
                let image = slot.image.as_ref()?;
                let width = 72. * image.width() as f32 / image.height() as f32;
                Some(
                    div()
                        .relative()
                        .w(px(width))
                        .h(px(72.))
                        .overflow_hidden()
                        .child(super::advanced::checker())
                        .child(xgpui_media::image_view(image.clone()).size_full().rounded(px(6.)))
                        .child(
                            text(&self.look, TextRole::Value, format!("{:02}", index + 1))
                                .absolute()
                                .bottom(px(3.))
                                .left(px(3.)),
                        ),
                )
            }),
        );
        let message = if lab.burst.is_empty() {
            "Start a burst to watch 12 fresh snapshots pass through bounded loading and uploads.".into()
        } else if !lab.admission.is_empty() {
            format!("Backpressure: {}. Completed previews retire to release capacity.", lab.admission)
        } else {
            "READY = host-submitted. DONE = a submitted preview retired to make room.".into()
        };
        let run = div().w_full().flex().flex_col().gap_3().child(chips).child(previews).child(text(
            &self.look,
            TextRole::Caption,
            message,
        ));
        let screen = LabScreen::new("media-resources", self.look.clone())
            .crumb("xgpui-media", "Resource Limits")
            .title("Resource Limits")
            .lede("Compare how upload limits spread twelve fresh images across frames.")
            .hero(Hero::stage(ComparePane::new("Upload run / 12 snapshots", run)))
            .caption(format!(
                "{} / {} submitted · {} uploads / frame · {} MiB / frame",
                lab.submitted(),
                lab.burst.len(),
                lab.count_limit,
                lab.byte_limit / MIB
            ))
            .notice(self.error.clone())
            .notice(lab.error());
        let screen = self.screen_choices(screen, &[Action::Sample, Action::Jobs, Action::Budget]);
        let screen = self.screen_actions(screen, &[Action::Open, Action::Reload, Action::Burst]);
        let usage = lab.loader.as_ref().ok().map(|loader| loader.upload_stats());
        let pool = lab.loader.as_ref().ok().map(|loader| loader.staging_pool_stats());
        let measured = f32::from(self.bounds[1].size.width);
        let preview_width = if measured > 1. { measured } else { 300. };
        let scale = self.display_scale(preview_width, 220., cx).unwrap_or(0.);
        let mut primary = self.camera_viewport(1, preview_width, 220., true, cx);
        if let (Some(image), Some(fixtures)) = (&lab.primary.image, &self.fixtures) {
            let sample = &fixtures.samples[self.selected];
            let width = sample.width as f32 * scale;
            let height = sample.height as f32 * scale;
            primary = primary.child(
                xgpui_media::image_view(image.clone())
                    .absolute()
                    .left(px((preview_width - width) * 0.5 + self.values[1]))
                    .top(px((220. - height) * 0.5 + self.values[2]))
                    .w(px(width))
                    .h(px(height))
                    .rounded(px(self.values[4]))
                    .opacity(self.values[3]),
            );
        }
        let mib = |bytes: usize| format!("{:.2} MiB", bytes as f64 / MIB as f64);
        let screen = screen.cards([
            LabCard::new("Upload policy").subtitle("New sources per frame")
                .values([("count", lab.count_limit.to_string()), ("bytes", mib(lab.byte_limit))]),
            LabCard::new("Run results").subtitle(if lab.burst.is_empty() { "Ready to run" } else if lab.burst.iter().any(|slot| slot.pending()) { "Running" } else { "Finished" })
                .values([("submitted", lab.submitted().to_string()), ("failed", lab.burst.iter().filter(|slot| slot.phase == Phase::Failed).count().to_string())]),
            LabCard::new("Primary image").subtitle(lab.primary.phase.label())
                .values([("payload", lab.primary.image.as_ref().map_or("—".into(), |image| mib(image.payload_bytes())))]),
        ]).panels(
            PanelShell::new("Pipeline / live").child(properties(&self.look, [
                ("Background staging workers", usage.map_or("—".into(), |usage| format!("{} / {}", usage.active_jobs, UploadLimits::default().max_jobs))),
                ("Staging reservations", usage.map_or("—".into(), |usage| mib(usage.staging_bytes))),
                ("Pool allocated / idle", pool.as_ref().map_or("—".into(), |pool| format!("{:.2} / {:.2} MiB", pool.allocated_bytes as f64 / MIB as f64, pool.cached_bytes as f64 / MIB as f64))),
                ("Buffer allocations / reuse", pool.as_ref().map_or("—".into(), |pool| format!("{} / {}", pool.allocations, pool.reuses))),
                ("Decode workers", MediaLimits::default().max_jobs.to_string()),
            ])),
            PanelShell::new("Retained source").child(primary),
        ).section(div().flex().flex_wrap().gap_2().children(lab.burst.iter().enumerate().filter_map(|(index, slot)| {
            slot.error.as_ref().map(|error| text(&self.look, TextRole::Caption, format!("Snapshot {:02}: {error}", index + 1)))
        })))
            .note("Click Burst ×12 to start a run. Changing an upload policy reloads the primary and clears the previous burst, as in the original media lab. Final previews remain visible; older ready previews retire only when waiting work needs capacity.")
            .credits("Adapted from xgpui/apps/xgpui-ext-lab/media-lab. READY means host submission, not GPU completion. Limits govern per-frame source count and aligned staging bytes; they do not measure GPU time or total memory. Background decode and staging keep the media library's default limits.");
        self.camera_screen(screen, cx)
    }
}

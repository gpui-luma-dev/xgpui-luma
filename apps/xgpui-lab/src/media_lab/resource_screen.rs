//! Bounded decode and upload admission, using the shared diagnostics layout.
use gpui::{Context, IntoElement, div, prelude::*, px};

use super::{Action, MIB, MediaPage};
use crate::screen::{ComparePane, Hero, LabCard, LabScreen, PanelShell, properties};

impl MediaPage {
    pub(super) fn render_resources(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let image = div().w_full().h(px(240.)).flex().items_center().justify_center().children(
            self.image
                .as_ref()
                .map(|image| xgpui_media::image_view(image.clone()).w_full().max_w(px(300.)).h(px(220.))),
        );
        let burst = div().w_full().min_h(px(240.)).flex().flex_wrap().items_center().gap_2().children(
            self.burst_images.iter().map(|image| xgpui_media::image_view(image.clone()).w(px(110.)).h(px(80.))),
        );
        let screen = LabScreen::new("media-resources", self.look.clone())
            .crumb("xgpui-media", "Resource Limits")
            .title("Resource Limits")
            .lede("Explore how limits on simultaneous jobs and memory affect a burst of image loads.")
            .hero(Hero::compare(
                ComparePane::new("Retained image", image),
                ComparePane::new("Burst in progress", burst),
            ))
            .caption(format!(
                "{} waiting · {} decoding · {} preparing · {} retired",
                self.waiting,
                self.burst_jobs.len(),
                self.burst_images.len(),
                self.completed
            ))
            .notice(self.error.as_ref().map(|error| format!("Media error: {error}")));
        let screen = self.screen_choices(screen, &[Action::Sample, Action::Jobs, Action::Budget]);
        let screen = self.screen_actions(screen, &[Action::Open, Action::Reload, Action::Burst]);
        let mut admission = vec![
            ("Decode jobs", self.job_limit.to_string()),
            ("Source budget", format!("{} MiB", [32, 64, 128][self.budget_index])),
            ("Per image", "16 MiB".into()),
        ];
        let mut staging = Vec::new();
        match &self.loader {
            Ok(loader) => {
                admission.push(("Upload admission", format!("{:?}", loader.upload_stats())));
                staging.push(("Pool", format!("{:?}", loader.staging_pool_stats())));
            }
            Err(error) => admission.push(("Loader error", error.to_string())),
        }
        let image = self.image.as_ref();
        let screen = screen.cards([
            LabCard::new("Queue").subtitle("Burst ×12").values([("waiting", self.waiting.to_string()), ("retired", self.completed.to_string())]),
            LabCard::new("Active work").values([("decode", self.burst_jobs.len().to_string()), ("prepare", self.burst_images.len().to_string())]),
            LabCard::new("Retained source").subtitle(image.map_or("Waiting".into(), |image| format!("{:?}", image.upload_status())))
                .values([("payload", image.map_or("—".into(), |image| format!("{:.2} MiB", image.payload_bytes() as f64 / MIB as f64)))]),
        ]).panels(
            PanelShell::new("Admission limits").child(properties(&self.look, admission)),
            PanelShell::new("Staging pool").child(properties(&self.look, staging)),
        ).note("Completed burst snapshots retire to release their source reservations. Changing a limit rebuilds the loader and clears the burst queue.")
            .credits("Ready means host submission, not GPU completion. These budgets bound payload reservations, not total process RSS or driver VRAM.");
        self.camera_screen(screen, cx)
    }
}

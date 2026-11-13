//! A shared immutable video frame, compared before and after sampling adjustments.
use gpui::{Context, IntoElement, Window, div, prelude::*, px};
use super::{Action, MediaPage, Parameter};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use xgpui_luma::infra::ElementExt;
use crate::screen::{ComparePane, Hero, LabScreen, PanelShell, properties};

impl MediaPage {
    pub(super) fn render_video(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if self.video.is_none() && !self.video_stopped {
            match super::video_source::Demo::new(self.format) {
                Ok(video) => self.video = Some(video),
                Err(error) => {
                    self.error = Some(error.to_string());
                    self.video_stopped = true;
                }
            }
        }
        self.sync_switches(cx);
        let mut screen = LabScreen::new("media-video", self.look.clone())
            .crumb("xgpui-media", "Video Frames")
            .title("Video Frames")
            .lede("Compare the same moving frame before and after image adjustments and blur.");
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            // Preserve the bounded blur target, including the compositor's halo allowance.
            let measured = f32::from(self.bounds[0].size.width);
            let pane_width = if measured > 1. { measured } else { 560. };
            let width = (1020. / window.scale_factor()
                - 2. * (16. + 8. / window.scale_factor()) * gpui::MAX_LAYER_GROUPS as f32)
                .clamp(80., 640.)
                .min(pane_width);
            let frame = self.video.as_ref().and_then(|video| video.stream.snapshot());
            let stats = self.video.as_ref().map(|video| video.stream.stats());
            if stats.as_ref().is_some_and(|stats| stats.playing && !stats.closed) {
                window.request_animation_frame();
            }
            let adjustments = self.adjustments(xgpui_media::pipeline::ImagePrimaries::Srgb);
            let blur = self.blur();
            let panes = [true, false].map(|adjusted| {
                let mut viewport = div()
                    .relative()
                    .w(px(width))
                    .h(px(width * 9. / 16.))
                    .flex_shrink_0()
                    .overflow_hidden()
                    .rounded(px(self.values[4]))
                    .child(super::advanced::checker());
                if let Some(frame) = &frame {
                    let view = xgpui_media::video_frame(frame.clone());
                    let view = match (&adjustments, &blur) {
                        (Ok(adjustments), Ok(blur)) if adjusted => view.adjustments(*adjustments).blur(blur.clone()),
                        _ => view,
                    };
                    viewport = viewport.child(view.size_full().rounded(px(self.values[4])).opacity(self.values[3]));
                }
                let index = usize::from(!adjusted);
                let owner = cx.entity();
                div()
                    .w_full()
                    .on_prepaint(move |bounds, _, cx| {
                        owner.update(cx, |this, cx| {
                            let resized = this.bounds[index].size.width != bounds.size.width;
                            this.bounds[index] = bounds;
                            if resized {
                                cx.notify();
                            }
                        });
                    })
                    .child(viewport)
            });
            let [adjusted, original] = panes;
            screen = screen
                .hero(Hero::compare(
                    ComparePane::new("Adjusted preview", adjusted).subtitle("Linear light + bounded blur"),
                    ComparePane::new("Original reference", original).subtitle("Same immutable frame"),
                ))
                .caption(format!(
                    "{} · timestamp {:.3} s · {} · blur {} px · {} quality",
                    self.format.label(),
                    frame.as_ref().map_or(0., |frame| frame.timestamp().as_secs_f64()),
                    if self.video_stopped {
                        "stopped"
                    } else if self.video_paused {
                        "paused"
                    } else {
                        "playing"
                    },
                    [0, 4, 8, 16][self.blur_index],
                    if self.full_quality { "full" } else { "adaptive" }
                ))
                .notice(
                    self.video.as_ref().and_then(|video| video.error()).map(|error| format!("Source error: {error}")),
                )
                .notice(adjustments.err().map(|error| format!("Adjustments unavailable: {error}")))
                .notice(blur.err().map(|error| format!("Blur unavailable: {error}")))
                .section_panel(PanelShell::new("Frame mailbox").child(properties(
                    &self.look,
                    [("State", stats.map_or("No source".into(), |stats| format!("{stats:?}")))],
                )));
            screen = self.screen_choices(screen, &[Action::Format, Action::Blur]);
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = window;
            screen = screen
                .hero(Hero::full(div().h(px(200.))))
                .caption("Native video source unavailable on this platform")
                .notice(Some("The generated source requires native macOS Metal or Windows D3D11."));
        }
        screen = self.screen_switches(screen, &[Action::Animate, Action::Quality]);
        if self.has_video() {
            screen = self.screen_switches(screen, &[Action::Pause]);
        }
        screen = self.screen_actions(screen, &[Action::Start, Action::Restart, Action::Reset]);
        if self.has_video() {
            screen = self.screen_actions(screen, &[Action::VideoBurst]);
        }
        let parameters = [
            Parameter::Opacity,
            Parameter::Radius,
            Parameter::Exposure,
            Parameter::LinearContrast,
            Parameter::LinearSaturation,
            Parameter::Vignette,
        ];
        screen.notice(self.error.as_ref().map(|error| format!("Video error: {error}")))
            .panels(self.parameter_panel("Frame parameters", &parameters), self.adjustment_chain())
            .note("Stop retains the last frame. Restart replaces the source. Pause freezes presentation while the producer can continue publishing to the bounded mailbox.")
            .credits("Generated moving BGRA fixture on macOS and Windows; macOS also supports opaque NV12. This is not a decoder or an audio/video clock. Blur stays within a bounded SDR target.")
    }
}

//! Wide Color Fills presentation; sample conversion stays in the gamut model.
use super::*;
use crate::screen::{ComparePane, Hero, LabScreen, PanelShell, TextRole, properties, text};

/// Linear Display P3 data colors, shared by the native and clipped reference paths.
pub(super) const FILL_SAMPLES: [(&str, [f64; 3]); 4] = [
    ("P3 Red", [1.0, 0.0, 0.0]),
    ("P3 Green", [0.0, 1.0, 0.0]),
    ("P3 Orange", [1.0, 0.25, 0.0]),
    ("P3 Magenta", [1.0, 0.0, 1.0]),
];

impl GamutPage {
    pub(super) fn presentation_panel(&self, display: &str, fallback: &str, window: &Window) -> PanelShell {
        let status = window.color_status();
        PanelShell::new("Window presentation")
            .subtitle("Negotiated output, not measured display fidelity")
            .child(properties(
                &self.look,
                [
                    (
                        "Intent",
                        if self.requested_wide {
                            "P3 (WideGamutSdr)"
                        } else {
                            "sRGB (Default)"
                        }
                        .into(),
                    ),
                    ("Accepted", format!("{:?}", status.requested())),
                    ("Effective", format!("{:?}", status.effective())),
                    ("Backend support", status.supports_wide_gamut_sdr().to_string()),
                    ("Display", display.to_owned()),
                    ("Fallback", fallback.to_owned()),
                ],
            ))
    }

    pub(super) fn fills_screen(&self, wide: bool, display: &str, fallback: &str, window: &Window) -> LabScreen {
        let screen = LabScreen::new("gamut-fills", self.look.clone())
            .crumb("xgpui-gamut", "Wide Color Fills")
            .title("Wide Color Fills")
            .lede("Compare vivid Display P3 colors with their clipped sRGB versions.")
            .caption(if wide {
                "4 colors · native linear Display P3 · clipped sRGB references"
            } else {
                "4 colors · sRGB fallback previews · native P3 output unavailable"
            })
            .section_panel(self.presentation_panel(display, fallback, window))
            .note("Both columns start from the same linear Display P3 samples. The sRGB references clip each converted linear channel to 0–1 before encoding.")
            .credits("Native P3 colors require a wide-gamut window and display. In fallback mode both columns show the same clipped sRGB previews; screenshots do not measure display fidelity.");
        match &self.samples {
            Err(error) => screen.notice(Some(format!("Samples unavailable: {error}"))),
            Ok(samples) => {
                let column = |native: bool| {
                    div().flex().flex_col().gap_3().children(samples.iter().enumerate().map(|(index, sample)| {
                        let color = sample.wide.clone();
                        let swatch = div()
                            .debug_selector(|| format!("gamut-fills-{}-{index}", if native { "p3" } else { "srgb" }))
                            .h(px(64.0))
                            .w_full()
                            .when(!native || !wide, |swatch| swatch.bg(sample.srgb))
                            .when(native && wide, |swatch| {
                                swatch.child(
                                    canvas(
                                        |_, _, _| (),
                                        move |bounds, _, window, _| {
                                            window.paint_custom(
                                                CustomGeometry::RoundedRect {
                                                    bounds,
                                                    corner_radii: Default::default(),
                                                },
                                                color.clone(),
                                            );
                                        },
                                    )
                                    .size_full(),
                                )
                            });
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(text(&self.look, TextRole::Detail, sample.label))
                            .child(swatch)
                    }))
                };
                screen.hero(Hero::compare(
                    ComparePane::new(if wide { "Display P3" } else { "P3 → sRGB fallback" }, column(true)),
                    ComparePane::new("Clipped sRGB", column(false)),
                ))
            }
        }
    }
}

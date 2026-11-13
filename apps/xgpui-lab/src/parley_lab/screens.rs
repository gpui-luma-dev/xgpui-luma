//! Screen composition; shaping, fallback resolution and CPU masks stay in the owner.
use super::*;
use crate::screen::{ComparePane, Hero, LabScreen, PanelShell, ParametersPanel, TextRole, properties, text};
impl ParleyPage {
    pub(super) fn backend_error(&self, id: &'static str, title: &'static str) -> LabScreen {
        LabScreen::new(id, self.look.clone())
            .crumb("xgpui-parley", title)
            .title(title)
            .notice(Some(format!("Parley initialization failed: {:?}", self.engine.as_ref().err())))
    }
    pub(super) fn row_screen(&self, cx: &mut Context<Self>) -> LabScreen {
        let (id, title, lede) = match self.active_tab.as_ref() {
            "Fonts & Fallback" => (
                "parley-fonts",
                "Fonts & Fallback",
                "Explore how a line finds the fonts it needs across different scripts.",
            ),
            "Rasterization" => (
                "parley-raster",
                "Rasterization",
                "Inspect the pixels produced from shaped text at different raster scales.",
            ),
            _ => (
                "parley-shaping",
                "Text Shaping",
                "Explore how letters become glyphs, clusters and resolved font runs.",
            ),
        };
        self.numbers[0].observe(self.font_size, cx);
        self.numbers[1].observe(self.raster_scale, cx);
        let mut screen = LabScreen::new(id, self.look.clone()).crumb("xgpui-parley", title).title(title).lede(lede)
            .caption(format!("{:.0} px text · {:.2}× CPU raster scale · isolated Parley backend", self.font_size, self.raster_scale))
            .panels(ParametersPanel::new("Text parameters").params(self.numbers.iter().take(if title == "Rasterization" { 2 } else { 1 }).map(|n|n.param())),
                PanelShell::new("Diagnostics").child(properties(&self.look, [
                    ("CPU run", if self.measuring { "Running" } else if self.report.is_some() { "Finished" } else { "Not measured" }.to_owned()),
                    ("Iterations", "20 warm samples / median".into()),
                    ("Seed size", "28 px (independent of preview)".into()),
                ])))
            .note("Preview controls rebuild the retained CPU masks. CPU diagnostics use the original font fixtures and a separate layout cache.")
            .credits("CPU layout and direct glyph rasterization only; excludes frame and GPU timing. Native reference rows use the shell’s existing backend. Mask subjects use fixed paper and ink in both themes.");
        for button in self.buttons.iter().take(4) {
            screen = screen.action(button.clone());
        }
        if let Some(error) = &self.child_error {
            screen = screen.notice(Some(format!("Native atlas window failed: {error}")));
        }
        if let Some(Err(error)) = &self.report {
            screen = screen.notice(Some(format!("Diagnostic failed: {error:#}")));
        }
        match self.rows.as_ref().expect("rows prepared before screen") {
            Err(error) => screen.notice(Some(format!("Text lab unavailable: {error:#}"))),
            Ok(rows) => {
                let first = if title == "Fonts & Fallback" { 2 } else { 0 };
                let row = &rows[first];
                screen = screen.hero(Hero::stage(ComparePane::new(row.label.clone(), self.mask(row))));
                for (index, row) in rows.iter().enumerate().skip(first) {
                    let mut panel = PanelShell::new(row.label.clone()).child(properties(
                        &self.look,
                        [
                            ("Source", row.text.clone()),
                            ("Layout", format!("{:.2} px · {} resolved faces", row.width - 4., row.faces)),
                            ("Runs / clusters", row.details.clone()),
                        ],
                    ));
                    if title == "Fonts & Fallback" {
                        panel = panel
                            .child(text(
                                &self.look,
                                TextRole::Detail,
                                format!(
                                    "Requested {} · weight {} · fallback {:?}",
                                    row.font.family, row.font.weight.0, row.font.fallbacks
                                ),
                            ))
                            .child(
                                div()
                                    .text_size(px(self.font_size))
                                    .font(row.font.clone())
                                    .child(format!("Native reference: {}", row.text)),
                            );
                    }
                    if index != first {
                        panel = panel.child(self.mask(row));
                    }
                    if title == "Rasterization" {
                        let pixels = row.image.size(0);
                        let bytes = row.image.as_bytes(0).unwrap();
                        let coverage = bytes.as_chunks::<4>().0.iter().filter(|p| p[3] > 0).count();
                        panel = panel.child(text(
                            &self.look,
                            TextRole::Value,
                            format!(
                                "{}×{} device px · {coverage} covered pixels · {} bytes",
                                i32::from(pixels.width),
                                i32::from(pixels.height),
                                bytes.len()
                            ),
                        ));
                    }
                    if let Some(Ok(report)) = &self.report {
                        let t = &report[index];
                        panel = panel.child(text(&self.look, TextRole::Value, format!("28 px seed · layout first/warm {:.1}/{:.2} µs · raster first/warm {:.1}/{:.1} µs · {:.2} px · {} faces · {} glyphs", t.layout_first,t.layout_warm,t.raster_first,t.raster_warm,t.width,t.faces,t.glyphs)));
                    }
                    screen = screen.section_panel(panel);
                }
                screen
            }
        }
    }
    fn mask(&self, row: &Row) -> impl IntoElement + use<> {
        div()
            .id(format!("parley-mask-{}", row.label))
            .w_full()
            .overflow_x_scroll()
            .bg(rgb(palette::PAPER))
            .p_3()
            .child(img(row.image.clone()).w(px(row.width)).h(px(row.height)))
    }
}

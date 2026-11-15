//! Source-style workbench controls and inspection readout using the Lab's SDK chrome.
use gpui::{prelude::*, Div, div, px};
use crate::gpu_interop::{Page, runtime::Snapshot};
use crate::screen::{TextRole, panel_surface, properties, text};

impl Page {
    pub(in crate::gpu_interop) fn workbench_controls(&self) -> Div {
        let mut controls = div()
            .debug_selector(|| "workbench-controls".into())
            .flex()
            .flex_col()
            .gap_3()
            .child(self.buttons[0].clone())
            .child(text(&self.look, TextRole::Label, "Colour"))
            .child(div().flex().flex_wrap().gap_2().children(self.swatches.iter().cloned()));
        for (label, slider) in [
            format!("Radius {:.0} px", self.settings.radius),
            format!("Opacity {:.0}%", self.settings.opacity * 100.),
            format!("Exposure {:+.1}", self.settings.exposure),
        ]
        .into_iter()
        .zip(&self.sliders)
        {
            controls = controls.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(text(&self.look, TextRole::Label, label))
                    .child(slider.clone()),
            );
        }
        controls = controls
            .children(self.choices.iter().cloned())
            .child(self.compare_control.clone())
            .child(self.pause.clone());
        controls.child(div().flex().flex_wrap().gap_2().children((1..6).map(|index| self.buttons[index].clone())))
    }

    pub(in crate::gpu_interop) fn inspection_properties(&self, snapshot: &Snapshot) -> Vec<(&'static str, String)> {
        let Some(found) = &snapshot.inspection_data else {
            return Vec::new();
        };
        let Some(pixel) = found.pixels.first() else {
            return Vec::new();
        };
        let published = pixel[..3]
            .iter()
            .map(|value| (value * self.settings.exposure.exp2()).clamp(0., 1.))
            .collect::<Vec<_>>();
        vec![
            ("Region", format!("4×4 at ({}, {})", found.x, found.y)),
            (
                "Revision",
                format!(
                    "{}{}",
                    found.revision,
                    if found.revision == snapshot.revision {
                        " (current)".into()
                    } else {
                        format!(" (now {})", snapshot.revision)
                    }
                ),
            ),
            ("Generation", found.generation.to_string()),
            ("Working linear P3", format!("{:.4} {:.4} {:.4} · α {:.2}", pixel[0], pixel[1], pixel[2], pixel[3])),
            (
                "Published",
                format!(
                    "{:.4} {:.4} {:.4} · exposure {:+.1}",
                    published[0], published[1], published[2], self.settings.exposure
                ),
            ),
        ]
    }

    pub(in crate::gpu_interop) fn inspection_view(&self, snapshot: &Snapshot) -> Div {
        let rows = self.inspection_properties(snapshot);
        panel_surface(&self.look)
            .debug_selector(|| "workbench-inspect".into())
            .min_w(px(0.))
            .child(text(&self.look, TextRole::Section, "Inspect"))
            .child(text(
                &self.look,
                TextRole::Caption,
                "Right-click the main canvas to inspect a 4×4 region. Values below describe its first pixel.",
            ))
            .when(!rows.is_empty(), |view| view.child(properties(&self.look, rows)))
            .when(snapshot.inspection.starts_with("Inspection unavailable"), |view| {
                view.child(text(&self.look, TextRole::Caption, snapshot.inspection.clone()))
            })
    }
}

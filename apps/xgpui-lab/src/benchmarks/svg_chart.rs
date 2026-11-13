//! Native in-app comparison plot and the original lab's large artwork previews.
use super::*;
use crate::{
    screen::{text, TextRole},
    theme::ColorVizLookExt,
};
const SERIES_COLORS: [u32; 6] = [0x3984d6, 0xd17b32, 0x35a779, 0xa16acb, 0xd45068, 0x79923d];
const PREVIEW_PAPER: u32 = 0xffffff;
#[derive(Debug)]
struct Series {
    id: String,
    name: String,
    points: Vec<(f64, f64)>,
}
fn series(cases: &[Value]) -> Vec<Series> {
    let mut all: Vec<Series> = Vec::new();
    for case in cases {
        let Some(id) = case["id"].as_str() else {
            continue;
        };
        let index = all.iter().position(|s| s.id == id).unwrap_or_else(|| {
            all.push(Series {
                id: id.into(),
                name: case["display_name"].as_str().unwrap_or(id).into(),
                points: vec![],
            });
            all.len() - 1
        });
        if let (Some(bytes), Some(ms)) = (case["budget_bytes"].as_u64(), case["raster"]["median_ms"].as_f64())
            && ms.is_finite()
            && ms >= 0.
            && case.get("error").is_none()
        {
            all[index].points.push((bytes as f64 / xgpui_svg::MIB as f64, ms));
        }
    }
    for s in &mut all {
        s.points.sort_by(|a, b| a.0.total_cmp(&b.0));
    }
    all
}
pub(super) fn gallery(
    look: &Look,
    cases: &[Value],
    images: &svg_table::Thumbnails,
    selected: Option<&Path>,
) -> impl IntoElement + use<> {
    let mut cards = div()
        .id("svg-comparison-artwork")
        .debug_selector(|| "svg-comparison-artwork".into())
        .flex()
        .flex_wrap()
        .gap_4();
    for s in series(cases) {
        let title = if s.id == "Selected SVG" {
            selected.and_then(Path::file_name).map_or(s.id.clone(), |n| n.to_string_lossy().into_owned())
        } else {
            s.id.clone()
        };
        let image = div()
            .w_full()
            .h(px(105.))
            .bg(rgb(PREVIEW_PAPER))
            .overflow_hidden()
            .when_some(images.get(&s.id).cloned(), |v, image| {
                v.child(img(image).size_full().object_fit(ObjectFit::Contain))
            });
        let card = radix::Card::new(look)
            .child(text(look, TextRole::Name, title))
            .child(image)
            .child(text(look, TextRole::Caption, "Compare 16 / 32 MiB at the same zoom"))
            .into_element()
            .w(px(180.))
            .max_w_full()
            .flex_shrink_0();
        cards = cards.child(card);
    }
    cards
}
pub(super) fn render(look: &Look, cases: &[Value]) -> impl IntoElement + use<> {
    let all = series(cases);
    let measured = all.iter().any(|s| !s.points.is_empty());
    let max = all.iter().flat_map(|s| s.points.iter().map(|p| p.1)).fold(0., f64::max);
    let ceiling = (max * 1.1).ceil().max(1.);
    let mut legend = div().flex().flex_wrap().gap_4();
    for (index, s) in all.iter().enumerate() {
        legend = legend.child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().size(px(10.)).bg(rgb(SERIES_COLORS[index % SERIES_COLORS.len()])))
                .child(text(look, TextRole::Caption, s.name.clone())),
        );
    }
    let chrome = look.chrome();
    let plot = canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let left = bounds.left() + px(16.);
            let right = bounds.right() - px(16.);
            let top = bounds.top() + px(8.);
            let bottom = bounds.bottom() - px(8.);
            let line = |a, b, color, width, window: &mut Window| {
                let mut path = PathBuilder::stroke(px(width));
                path.move_to(a);
                path.line_to(b);
                if let Ok(path) = path.build() {
                    window.paint_path(path, color);
                }
            };
            for tick in 0..=4 {
                let y = top + (bottom - top) * (tick as f32 / 4.);
                line(point(left, y), point(right, y), chrome.border, 1., window);
            }
            for (index, s) in all.iter().enumerate() {
                let color = rgb(SERIES_COLORS[index % SERIES_COLORS.len()]);
                let mut previous = None;
                for (budget, ms) in &s.points {
                    let p = point(
                        left + (right - left) * ((*budget - 16.) / 16.) as f32,
                        bottom - (bottom - top) * (*ms / ceiling) as f32,
                    );
                    if let Some(a) = previous {
                        line(a, p, color.into(), 2., window);
                    }
                    window.paint_quad(fill(Bounds::new(p - point(px(3.), px(3.)), size(px(6.), px(6.))), color));
                    previous = Some(p);
                }
            }
        },
    )
    .w_full()
    .h(px(280.));
    let y = div()
        .w(px(72.))
        .h(px(280.))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .justify_between()
        .children((0..=4).map(|i| {
            text(
                look,
                TextRole::Value,
                if measured {
                    format!("{:.1}", ceiling * (4 - i) as f64 / 4.)
                } else {
                    "—".into()
                },
            )
        }));
    div().id("svg-budget-chart").debug_selector(||"svg-budget-chart".into()).w_full().flex().flex_col().gap_3()
        .child(text(look,TextRole::Section,"Median raster time by budget"))
        .child(text(look,TextRole::Caption,"Raster time (ms) · lower is faster"))
        .child(legend)
        .child(div().flex().gap_2().child(y).child(div().flex_1().min_w_0().flex().flex_col().gap_2().child(plot)
            .child(div().flex().justify_between().px_4().child(text(look,TextRole::Value,"16 MiB")).child(text(look,TextRole::Value,"32 MiB")))))
        .child(text(look,TextRole::Caption,if measured {"Raster budget (MiB). Lines compare the same artwork and zoom; check actual pixels and capping in the table."}else{"Raster budget (MiB). Waiting for measurements; pending and failed cases are not plotted as zero."}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn chart_uses_budget_as_x_and_keeps_pending_cases_out_of_the_data() {
        let values = vec![
            json!({"id":"a","budget_bytes":32*xgpui_svg::MIB,"raster":{"median_ms":2.}}),
            json!({"id":"a","budget_bytes":16*xgpui_svg::MIB,"raster":{"median_ms":1.}}),
            json!({"id":"b","budget_bytes":16*xgpui_svg::MIB,"outcome":"pending"}),
        ];
        let lines = series(&values);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].points, vec![(16., 1.), (32., 2.)]);
        assert!(lines[1].points.is_empty());
    }
}

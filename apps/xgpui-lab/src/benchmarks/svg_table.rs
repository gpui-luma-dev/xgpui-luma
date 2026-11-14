//! Table tab: one row per artwork, zoom and budget, grouped by artwork.
use super::*;
use super::svg_model;
use xgpui_luma::controls::table::Table;
use super::svg_badge::{Status, status_badge};
const THUMBNAIL_PAPER: u32 = 0xffffff;
const ROW_HEIGHT: f32 = 40.;
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Row {
    /// Only the first row of each artwork group shows the artwork.
    first: bool,
    thumbnail: Option<Arc<RenderImage>>,
    name: String,
    source: String,
    zoom: String,
    budget: String,
    output: String,
    capped: String,
    parse: String,
    raster: String,
    max: String,
    cost: String,
    memory: String,
    outcome: String,
}
fn ms(value: Option<f64>, digits: usize) -> String {
    value.map_or_else(|| "—".into(), |n| format!("{n:.digits$}"))
}
/// Flatten job reports into case rows; reports without cases become one row each.
pub(super) fn flatten(results: &[Value]) -> Vec<Value> {
    results
        .iter()
        .flat_map(|result| match result["cases"].as_array() {
            Some(cases) => cases.clone(),
            None => {
                let mut row = result.clone();
                if row.get("id").is_none() {
                    row["id"] = result["label"].clone();
                }
                vec![row]
            }
        })
        .collect()
}
pub(crate) fn rows(results: &[Value]) -> Vec<Row> {
    let mut previous: Option<String> = None;
    svg_model::cases(&flatten(results))
        .into_iter()
        .map(|case| {
            let first = previous.as_deref() != Some(case.id.as_str());
            previous = Some(case.id.clone());
            let m = case.measure;
            Row {
                first,
                thumbnail: None,
                name: case.name.clone(),
                source: case.id.clone(),
                zoom: if case.zoom.is_finite() {
                    svg_model::zoom_label(case.zoom)
                } else {
                    "—".into()
                },
                budget: if case.budget_mib > 0 {
                    format!("{} MiB", case.budget_mib)
                } else {
                    "—".into()
                },
                output: m.map_or_else(|| "—".into(), |m| format!("{}×{}", m.width, m.height)),
                capped: m.map_or("—", |m| if m.capped { "yes" } else { "no" }).into(),
                parse: ms(m.map(|m| m.parse_ms), 2),
                raster: ms(m.map(|m| m.raster_ms), 1),
                max: ms(m.and_then(|m| m.raster_max_ms), 1),
                cost: ms(m.and_then(|m| m.ms_per_megapixel()), 1),
                memory: ms(m.and_then(|m| m.rss_mib), 0),
                outcome: case.outcome.label().into(),
            }
        })
        .collect()
}
pub(crate) type Thumbnails = std::collections::HashMap<String, Arc<RenderImage>>;
pub(crate) fn with_thumbnails(mut rows: Vec<Row>, images: &Thumbnails) -> Vec<Row> {
    for row in rows.iter_mut().filter(|row| row.first) {
        // Finished measurements keep the artwork captured for that run.
        let settled = !matches!(row.outcome.as_str(), "pending" | "running");
        if row.thumbnail.is_some() && settled {
            continue;
        }
        row.thumbnail = images.get(&row.source).cloned();
    }
    rows
}
/// One image for every row, e.g. the Scaling page's single artwork.
pub(crate) fn with_image(mut rows: Vec<Row>, image: Option<Arc<RenderImage>>) -> Vec<Row> {
    for row in &mut rows {
        row.thumbnail = image.clone();
    }
    rows
}
/// Pending rows for the Scaling page's five samples at one zoom.
pub(crate) fn samples(zoom: f32) -> Vec<Value> {
    (1..=5).map(|n| json!({"id":format!("Sample {n}"),"zoom":zoom,"outcome":"pending"})).collect()
}
/// Pending rows for every case the selected jobs will run.
pub(super) fn plan(jobs: &[Job], zooms: &[f32], budgets: &[usize]) -> Vec<Value> {
    jobs.iter()
        .flat_map(|job| match job {
            Job::SvgCorpus => crate::svg_lab::benchmarks::planned_cases(zooms, budgets),
            Job::SvgBuiltin | Job::SvgSelected => {
                crate::svg_lab::benchmarks::planned_rows(json!({"id":job.label(),"job":job.id()}), zooms, budgets)
            }
            _ => vec![],
        })
        .collect()
}
pub(crate) fn view(table: Table<Row>) -> impl IntoElement {
    div()
        .id("svg-comparison-table-scroll")
        .debug_selector(|| "svg-comparison-table-scroll".into())
        .w_full()
        .min_w_0()
        .overflow_x_scroll()
        .child(div().w_full().min_w(px(1040.)).child(table))
}
/// Right-aligned number cell, so digits line up down a column.
fn number(value: &str) -> gpui::Div {
    div().w_full().flex().justify_end().child(value.to_owned())
}
pub(crate) fn build<M: 'static>(look: &Look, cx: &mut Context<M>) -> Table<Row> {
    let capped = crate::screen::data::color(crate::screen::data::CAPPED);
    let status_look = look.clone();
    radix::table_builder::<Row>(look, "svg-benchmark-results")
        .visible_rows(14)
        .visible_row_height(ROW_HEIGHT)
        .grid_columns()
        .column_fixed("", 56., |r| {
            let image = r.thumbnail.clone().filter(|_| r.first);
            div()
                .w(px(40.))
                .h(px(30.))
                .overflow_hidden()
                .when(image.is_some(), |cell| cell.bg(gpui::rgb(THUMBNAIL_PAPER)))
                .when_some(image, |cell, image| cell.child(img(image).size_full().object_fit(ObjectFit::Contain)))
        })
        .column_fill("Artwork", |r| if r.first { r.name.clone() } else { String::new() })
        .column_fixed("Zoom", 64., |r| number(&r.zoom))
        .column_fixed("Budget", 80., |r| number(&r.budget))
        .column_fixed("Output px", 110., |r| number(&r.output))
        .column_fixed("Capped", 80., move |r| {
            div().w_full().flex().justify_center().map(|cell| match r.capped.as_str() {
                "yes" => cell.text_color(capped).child("yes"),
                other => cell.opacity(0.55).child(other.to_owned()),
            })
        })
        .column_fixed("Parse ms", 84., |r| number(&r.parse))
        .column_fixed("Raster ms", 90., |r| number(&r.raster))
        .column_fixed("Max ms", 80., |r| number(&r.max))
        .column_fixed("ms/Mpx", 76., |r| number(&r.cost))
        .column_fixed("RSS MiB", 80., |r| number(&r.memory))
        .column_fixed("Status", 96., move |r| {
            let status = match r.outcome.as_str() {
                "done" => Some(Status::Done),
                "running" => Some(Status::Running),
                "failed" => Some(Status::Failed),
                _ => None,
            };
            div().w_full().flex().justify_center().map(|cell| match status {
                Some(status) => cell.child(status_badge(&status_look, r.outcome.clone(), status)),
                None => cell.opacity(0.55).child(r.outcome.clone()),
            })
        })
        .finish()
        .spawn(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg_lab::benchmarks::{COMPARISON_BUDGETS, SWEEP_ZOOMS};
    #[::core::prelude::v1::test]
    fn artwork_groups_show_their_preview_once_and_keep_it_after_finishing() {
        let images = crate::svg_lab::benchmarks::thumbnails(None, None);
        assert_eq!(images.len(), 5, "four corpus artworks and the built-in scene");
        let planned = with_thumbnails(rows(&plan(&[Job::SvgCorpus], &SWEEP_ZOOMS, &COMPARISON_BUDGETS)), &images);
        assert_eq!(planned.len(), 40);
        for group in planned.chunks(10) {
            assert!(group[0].first && group[1..].iter().all(|row| !row.first));
            let image = group[0].thumbnail.as_ref().expect("corpus preview");
            let size = image.size(0);
            assert!(i32::from(size.width) <= 360 && i32::from(size.height) <= 360);
            assert!(group[1..].iter().all(|row| row.thumbnail.is_none()));
            assert!(group.iter().all(|row| row.outcome == "pending"));
        }
        let finished = with_thumbnails(
            rows(&[json!({"id":planned[0].name,"zoom":1,"budget_bytes":16*xgpui_svg::MIB,"raster":{"median_ms":1.2}})]),
            &images,
        );
        let original = finished[0].thumbnail.clone().unwrap();
        let mut changed = images.clone();
        changed.insert(finished[0].name.clone(), images["Built-in SVG"].clone());
        let retained = with_thumbnails(finished, &changed);
        assert!(
            Arc::ptr_eq(retained[0].thumbnail.as_ref().unwrap(), &original),
            "changing the next input must not relabel a completed result's image"
        );
    }
    #[::core::prelude::v1::test]
    fn plans_cover_every_artwork_zoom_and_budget_in_run_order() {
        let corpus = plan(&[Job::SvgCorpus], &SWEEP_ZOOMS, &COMPARISON_BUDGETS);
        assert_eq!(corpus.len(), 40);
        let order: Vec<_> = corpus[..4]
            .iter()
            .map(|r| (r["zoom"].as_f64().unwrap(), r["budget_bytes"].as_u64().unwrap()))
            .collect();
        let (low, high) = (COMPARISON_BUDGETS[0] as u64, COMPARISON_BUDGETS[1] as u64);
        assert_eq!(order, vec![(1., low), (1., high), (2., low), (2., high)]);
        assert!(corpus.iter().all(|r| r["input_bytes"].as_u64().is_some_and(|b| b > 0)));
        assert_eq!(plan(&[Job::SvgCorpus], &[4.], &COMPARISON_BUDGETS[..1]).len(), 4);
        assert_eq!(plan(&Suite::Svg.jobs(), &SWEEP_ZOOMS, &COMPARISON_BUDGETS).len(), 60);
        assert!(rows(&corpus).iter().all(|row| row.outcome == "pending" && row.raster == "—"));
    }
    #[::core::prelude::v1::test]
    fn tables_flatten_cases_and_keep_failures_and_missing_metrics_visible() {
        let rows = rows(&[
            json!({"label":"corpus","cases":[
                {"id":"art","zoom":4,"budget_bytes":16*xgpui_svg::MIB,"parse":{"median_ms":1.25},"raster":{"median_ms":2.5,"max_ms":2.75},
                 "pixel_width":1000,"pixel_height":500,"sampled_process_rss_peak_bytes":1048576},
                {"id":"bad","zoom":8,"error":"failed worker"}
            ]}),
            json!({"id":"Sample 1","parse_ms":0.125,"raster_ms":0.25,"pixel_width":10,"pixel_height":10}),
        ]);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].parse, "1.25");
        assert_eq!(rows[0].output, "1000×500");
        assert_eq!(rows[0].cost, "5.0");
        assert_eq!(rows[0].max, "2.8");
        assert_eq!(rows[0].memory, "1");
        assert_eq!(rows[1].outcome, "failed");
        assert_eq!(rows[1].raster, "—");
        assert_eq!(rows[2].parse, "0.12");
    }
}

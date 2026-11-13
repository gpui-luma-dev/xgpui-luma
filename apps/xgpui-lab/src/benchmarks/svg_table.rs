//! Numeric rows for comparing artwork/zoom results without reading report files.
use super::*;
use xgpui_luma::controls::table::Table;
const THUMBNAIL_PAPER: u32 = 0xffffff;
#[derive(Clone, Debug)]
pub(crate) struct Row {
    thumbnail: Option<Arc<RenderImage>>,
    name: String,
    source: String,
    zoom: String,
    budget: String,
    parse: String,
    raster: String,
    pixels: String,
    capped: String,
    memory: String,
    outcome: String,
}
fn ms(v: &Value) -> String {
    v.as_f64().map_or_else(|| "—".into(), |n| format!("{n:.3}"))
}
pub(crate) fn rows(results: &[Value]) -> Vec<Row> {
    results
        .iter()
        .flat_map(|result| {
            let cases = result["cases"].as_array().map_or_else(|| vec![result], |cases| cases.iter().collect());
            cases
                .into_iter()
                .map(|c| Row {
                    thumbnail: None,
                    name: c["display_name"]
                        .as_str()
                        .or(c["id"].as_str())
                        .or(result["label"].as_str())
                        .unwrap_or("SVG")
                        .into(),
                    source: c["id"].as_str().or(result["label"].as_str()).unwrap_or("SVG").into(),
                    zoom: c["zoom"].as_f64().map_or_else(|| "—".into(), |n| format!("{n:.2}×")),
                    budget: c["budget_bytes"]
                        .as_u64()
                        .map_or_else(|| "—".into(), |bytes| format!("{} MiB", bytes / (xgpui_svg::MIB as u64))),
                    parse: ms(c.get("parse_ms").unwrap_or(&c["parse"]["median_ms"])),
                    raster: ms(c.get("raster_ms").unwrap_or(&c["raster"]["median_ms"])),
                    pixels: match (c["pixel_width"].as_u64(), c["pixel_height"].as_u64()) {
                        (Some(w), Some(h)) => format!("{w}×{h}"),
                        _ => "—".into(),
                    },
                    capped: c["resolution_limited"].as_bool().map_or("—", |v| if v { "yes" } else { "no" }).into(),
                    memory: c["sampled_process_rss_peak_bytes"]
                        .as_f64()
                        .map_or_else(|| "—".into(), |n| format!("{:.1}", n / 1048576.)),
                    outcome: c["outcome"]
                        .as_str()
                        .unwrap_or_else(|| {
                            if c.get("error").is_some() {
                                "failed"
                            } else {
                                "completed"
                            }
                        })
                        .into(),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
pub(crate) type Thumbnails = std::collections::HashMap<String, Arc<RenderImage>>;
pub(crate) fn with_thumbnails(mut rows: Vec<Row>, images: &Thumbnails) -> Vec<Row> {
    for row in &mut rows {
        // Completed measurements keep the artwork captured for that run.
        if row.thumbnail.is_some() && !matches!(row.outcome.as_str(), "pending" | "running") {
            continue;
        }
        row.thumbnail = images.get(&row.source).cloned();
    }
    rows
}
pub(crate) fn with_image(mut rows: Vec<Row>, image: Option<Arc<RenderImage>>) -> Vec<Row> {
    for row in &mut rows {
        row.thumbnail = image.clone();
    }
    rows
}
pub(crate) fn samples(zoom: f32) -> Vec<Value> {
    (1..=5).map(|n| json!({"id":format!("Sample {n}"),"zoom":zoom,"outcome":"pending"})).collect()
}
pub(super) fn plan(jobs: &[Job], zoom: f32) -> Vec<Value> {
    jobs.iter()
        .flat_map(|job| match job {
            Job::SvgCorpus => crate::svg_lab::benchmarks::planned_cases(zoom),
            Job::SvgBuiltin | Job::SvgSelected => {
                crate::svg_lab::benchmarks::COMPARISON_BUDGETS.map(|budget|json!({"id":job.label(),"job":job.id(),"zoom":zoom,"budget_bytes":budget,"outcome":"pending"})).to_vec()
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
pub(crate) fn build<M: 'static>(look: &Look, cx: &mut Context<M>) -> Table<Row> {
    radix::table_builder::<Row>(look, "svg-benchmark-results")
        .visible_rows(10)
        .visible_row_height(68.0_f32)
        .grid_columns()
        .column_fixed("Preview", 80., |r| {
            div()
                .w(px(64.))
                .h(px(52.))
                .overflow_hidden()
                .when(r.thumbnail.is_some(), |cell| cell.bg(gpui::rgb(THUMBNAIL_PAPER)))
                .when_some(r.thumbnail.clone(), |cell, image| {
                    cell.child(img(image).size_full().object_fit(ObjectFit::Contain))
                })
                .when(r.thumbnail.is_none(), |cell| cell.child("—"))
        })
        .column_fill("Artwork", |r| r.name.clone())
        .column_fixed("Budget", 88., |r| r.budget.clone())
        .column_fixed("Zoom", 72., |r| r.zoom.clone())
        .column_fixed("Parse ms", 90., |r| r.parse.clone())
        .column_fixed("Raster ms", 90., |r| r.raster.clone())
        .column_fixed("Pixels", 110., |r| r.pixels.clone())
        .column_fixed("Capped", 72., |r| r.capped.clone())
        .column_fixed("RSS MiB", 90., |r| r.memory.clone())
        .column_fixed("Status", 100., |r| r.outcome.clone())
        .finish()
        .spawn(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn pending_cases_have_small_cached_previews_shared_across_zoom_rows() {
        let images = crate::svg_lab::benchmarks::thumbnails(None, None);
        assert_eq!(images.len(), 5, "four corpus artworks and the built-in scene");
        let planned = rows(&plan(&[Job::SvgCorpus], 4.));
        let previewed = with_thumbnails(planned, &images);
        assert_eq!(previewed.len(), 8);
        for group in previewed.chunks(2) {
            let image = group[0].thumbnail.as_ref().expect("corpus preview");
            let size = image.size(0);
            assert!(i32::from(size.width) <= 360 && i32::from(size.height) <= 360);
            assert!(image.as_bytes(0).unwrap().as_chunks::<4>().0.iter().any(|p| p[3] > 0));
            for row in group {
                assert_eq!(row.outcome, "pending");
                assert!(Arc::ptr_eq(image, row.thumbnail.as_ref().unwrap()));
            }
        }
        let finished =
            with_thumbnails(rows(&[json!({"id":previewed[0].name,"zoom":1,"raster":{"median_ms":1.2}})]), &images);
        assert!(Arc::ptr_eq(finished[0].thumbnail.as_ref().unwrap(), previewed[0].thumbnail.as_ref().unwrap()));
        let original = finished[0].thumbnail.as_ref().unwrap().clone();
        let mut changed = images.clone();
        changed.insert(finished[0].name.clone(), images["Built-in SVG"].clone());
        let retained = with_thumbnails(finished, &changed);
        assert!(
            Arc::ptr_eq(retained[0].thumbnail.as_ref().unwrap(), &original),
            "changing the next input must not relabel a completed result's image"
        );
    }
    #[::core::prelude::v1::test]
    fn corpus_and_run_all_have_a_complete_pending_schedule() {
        let corpus = plan(&[Job::SvgCorpus], 4.);
        assert_eq!(corpus.len(), 8);
        assert!(corpus.iter().all(|r| r["outcome"] == "pending"));
        let identities: std::collections::HashSet<_> =
            corpus.iter().map(|r| (r["id"].as_str().unwrap(), r["budget_bytes"].as_u64().unwrap())).collect();
        assert_eq!(identities.len(), 8);
        assert_eq!(plan(&Suite::Svg.jobs(), 4.).len(), 12);
        assert!(rows(&corpus).iter().all(|row| row.outcome == "pending" && row.raster == "—"));
    }
    #[::core::prelude::v1::test]
    fn tables_flatten_cases_and_keep_failures_and_missing_metrics_visible() {
        let rows = rows(&[
            json!({"label":"corpus","cases":[
                {"id":"art","zoom":4,"parse":{"median_ms":1.25},"raster":{"median_ms":2.5},"pixel_width":640,"pixel_height":480,"sampled_process_rss_peak_bytes":1048576},
                {"id":"bad","zoom":8,"error":"failed worker"}
            ]}),
            json!({"id":"Sample 1","parse_ms":0.125,"raster_ms":0.25}),
        ]);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].parse, "1.250");
        assert_eq!(rows[0].pixels, "640×480");
        assert_eq!(rows[0].memory, "1.0");
        assert_eq!(rows[1].outcome, "failed");
        assert_eq!(rows[1].raster, "—");
        assert_eq!(rows[2].parse, "0.125");
    }
}

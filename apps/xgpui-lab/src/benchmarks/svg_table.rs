//! Numeric rows for comparing artwork/zoom results without reading report files.
use super::*;
use xgpui_luma::controls::table::Table;
#[derive(Clone, Debug)]
pub(crate) struct Row {
    name: String,
    zoom: String,
    parse: String,
    raster: String,
    pixels: String,
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
                    name: c["id"].as_str().or(result["label"].as_str()).unwrap_or("SVG").into(),
                    zoom: c["zoom"].as_f64().map_or_else(|| "—".into(), |n| format!("{n:.2}×")),
                    parse: ms(c.get("parse_ms").unwrap_or(&c["parse"]["median_ms"])),
                    raster: ms(c.get("raster_ms").unwrap_or(&c["raster"]["median_ms"])),
                    pixels: match (c["pixel_width"].as_u64(), c["pixel_height"].as_u64()) {
                        (Some(w), Some(h)) => format!("{w}×{h}"),
                        _ => "—".into(),
                    },
                    memory: c["sampled_process_rss_peak_bytes"]
                        .as_f64()
                        .map_or_else(|| "—".into(), |n| format!("{:.1}", n / 1048576.)),
                    outcome: if c.get("error").is_some() {
                        "failed"
                    } else {
                        c["outcome"].as_str().unwrap_or("completed")
                    }
                    .into(),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
pub(crate) fn build<M: 'static>(look: &Look, cx: &mut Context<M>) -> Table<Row> {
    radix::table_builder::<Row>(look, "svg-benchmark-results")
        .visible_rows(10)
        .grid_columns()
        .column_fill("Artwork", |r| r.name.clone())
        .column_fixed("Zoom", 72., |r| r.zoom.clone())
        .column_fixed("Parse ms", 90., |r| r.parse.clone())
        .column_fixed("Raster ms", 90., |r| r.raster.clone())
        .column_fixed("Pixels", 110., |r| r.pixels.clone())
        .column_fixed("RSS MiB", 90., |r| r.memory.clone())
        .column_fixed("Outcome", 100., |r| r.outcome.clone())
        .finish()
        .spawn(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
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

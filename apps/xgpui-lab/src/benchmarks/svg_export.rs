//! Export a run: `report.json` (replays with `--svg-results`) and a CSV of the cases for spreadsheets.
use super::*;
use super::svg_model;

/// The cases plus what they were measured with. `cases` is flat, so the file replays as is.
pub(super) fn report(cases: &[Value], zooms: &[f32], budgets: &[usize]) -> Value {
    let exported = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    json!({
        "cases": cases,
        "profile": profile(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "zooms": zooms,
        "budget_bytes": budgets,
        "exported_unix_seconds": exported,
    })
}

const COLUMNS: [&str; 17] = [
    "artwork",
    "zoom",
    "budget_mib",
    "output_width",
    "output_height",
    "output_mpx",
    "requested_width",
    "requested_height",
    "capped",
    "parse_ms",
    "raster_ms",
    "raster_max_ms",
    "ms_per_mpx",
    "rss_mib",
    "input_bytes",
    "outcome",
    "error",
];

/// Quote a field when it holds a comma, quote or line break (RFC 4180).
fn field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn number(value: Option<f64>) -> String {
    value.map_or_else(String::new, |n| format!("{n:.3}"))
}

/// One row per case with plain numbers: no units, and empty cells where a value is missing.
pub(super) fn csv(cases: &[Value]) -> String {
    let mut out = COLUMNS.join(",");
    out.push('\n');
    for case in svg_model::cases(cases) {
        let m = case.measure.as_ref();
        let row = [
            case.id.clone(),
            if case.zoom.is_finite() {
                case.zoom.to_string()
            } else {
                String::new()
            },
            if case.budget_mib > 0 {
                case.budget_mib.to_string()
            } else {
                String::new()
            },
            m.map_or_else(String::new, |m| m.width.to_string()),
            m.map_or_else(String::new, |m| m.height.to_string()),
            number(m.map(|m| m.megapixels())),
            m.and_then(|m| m.requested).map_or_else(String::new, |(w, _)| w.to_string()),
            m.and_then(|m| m.requested).map_or_else(String::new, |(_, h)| h.to_string()),
            m.map_or_else(String::new, |m| m.capped.to_string()),
            number(m.map(|m| m.parse_ms)),
            number(m.map(|m| m.raster_ms)),
            number(m.and_then(|m| m.raster_max_ms)),
            number(m.and_then(|m| m.ms_per_megapixel())),
            number(m.and_then(|m| m.rss_mib)),
            case.input_bytes.map_or_else(String::new, |b| b.to_string()),
            case.outcome.label().to_owned(),
            case.error.clone().unwrap_or_default(),
        ];
        out.push_str(&row.iter().map(|value| field(value)).collect::<Vec<_>>().join(","));
        out.push('\n');
    }
    out
}

/// Write `<path>.json` and `<path>.csv` side by side and return both paths.
pub(super) fn write(path: &Path, cases: &[Value], zooms: &[f32], budgets: &[usize]) -> Result<(PathBuf, PathBuf)> {
    let (json_path, csv_path) = (path.with_extension("json"), path.with_extension("csv"));
    std::fs::write(&json_path, serde_json::to_vec_pretty(&report(cases, zooms, budgets))?)?;
    std::fs::write(&csv_path, csv(cases))?;
    Ok((json_path, csv_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn csv_has_plain_numbers_blank_gaps_and_quoted_errors() {
        let cases = vec![
            json!({"id":"camera","zoom":4,"budget_bytes":16*xgpui_svg::MIB,"parse":{"median_ms":1.25},
                   "raster":{"median_ms":2.5,"max_ms":2.75},"pixel_width":1000,"pixel_height":500,"input_bytes":130}),
            json!({"id":"bad","zoom":8,"budget_bytes":32*xgpui_svg::MIB,"error":"worker said \"no\", twice"}),
        ];
        let text = csv(&cases);
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].split(',').count(), COLUMNS.len());
        assert!(lines[1].starts_with("camera,4,16,1000,500,0.500,"), "{}", lines[1]);
        assert!(lines[1].contains(",1.250,2.500,2.750,5.000,"), "{}", lines[1]);
        assert!(lines[1].ends_with(",130,done,"), "{}", lines[1]);
        assert!(lines[2].starts_with("bad,8,32,,,,"), "{}", lines[2]);
        assert!(lines[2].ends_with(",failed,\"worker said \"\"no\"\", twice\""), "{}", lines[2]);
    }
    #[::core::prelude::v1::test]
    fn exported_reports_replay_into_the_same_cases() {
        let dir = tempfile::tempdir().unwrap();
        let cases = svg_model::sample_cases();
        let (json_path, csv_path) = write(&dir.path().join("run"), &cases, &[1., 2.], &[16 * xgpui_svg::MIB]).unwrap();
        assert_eq!(json_path.file_name().unwrap(), "run.json");
        assert_eq!(std::fs::read_to_string(csv_path).unwrap().lines().count(), cases.len() + 1);
        let report: Value = serde_json::from_slice(&std::fs::read(json_path).unwrap()).unwrap();
        // serde_json may round the last float digit, so compare with the cases after one JSON pass.
        let expected: Vec<Value> = serde_json::from_slice(&serde_json::to_vec(&cases).unwrap()).unwrap();
        assert_eq!(svg_table::flatten(report["cases"].as_array().unwrap()), expected);
        assert_eq!(report["profile"], profile());
    }
    #[::core::prelude::v1::test]
    fn command_line_summaries_write_csv_for_svg_runs_only() {
        let dir = tempfile::tempdir().unwrap();
        let corpus = json!({"job":"corpus","cases":svg_model::sample_cases()});
        summary(dir.path(), std::slice::from_ref(&corpus), false).unwrap();
        let text = std::fs::read_to_string(dir.path().join("cases.csv")).unwrap();
        assert_eq!(text.lines().count(), corpus["cases"].as_array().unwrap().len() + 1);
        let other = tempfile::tempdir().unwrap();
        summary(other.path(), &[json!({"job":"paint","outcome":"completed"})], false).unwrap();
        assert!(!other.path().join("cases.csv").exists());
    }
}

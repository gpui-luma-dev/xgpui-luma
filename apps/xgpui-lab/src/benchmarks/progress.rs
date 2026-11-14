//! Shared progress snapshots; worker stage messages travel on stderr, never files.
use super::*;
use std::sync::Mutex;
#[derive(Clone)]
pub(crate) struct Progress {
    state: Arc<Mutex<State>>,
    started: Instant,
    worker: bool,
}
#[derive(Clone)]
struct State {
    case: String,
    stage: String,
    case_started: Instant,
    completed: Vec<Value>,
}
impl Default for Progress {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                case: "Preparing".into(),
                stage: String::new(),
                case_started: Instant::now(),
                completed: vec![],
            })),
            started: Instant::now(),
            worker: false,
        }
    }
}
impl Progress {
    pub(crate) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }
    pub(crate) fn elapsed(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    pub(crate) fn worker() -> Self {
        Self { worker: true, ..Default::default() }
    }
    pub(crate) fn case(&self, label: impl Into<String>) {
        if let Ok(mut s) = self.state.lock() {
            s.case = label.into();
            s.stage = "Starting".into();
            s.case_started = Instant::now();
        }
    }
    pub(crate) fn stage(&self, label: impl Into<String>) {
        let label = label.into();
        if self.worker {
            eprintln!("XGPUI_SVG_PROGRESS {label}");
        }
        if let Ok(mut s) = self.state.lock() {
            s.stage = label;
        }
    }
    pub(crate) fn plan(&self, rows: Vec<Value>) {
        if let Ok(mut s) = self.state.lock() {
            s.completed = rows;
        }
    }
    pub(crate) fn running(&self, id: &str, zoom: f64) {
        if let Ok(mut s) = self.state.lock()
            && let Some(row) = s.completed.iter_mut().find(|r| r["id"] == id && r["zoom"].as_f64() == Some(zoom))
        {
            row["outcome"] = json!("running");
        }
    }
    pub(crate) fn running_budget(&self, id: &str, zoom: f64, budget: usize) {
        if let Ok(mut s) = self.state.lock()
            && let Some(row) = s.completed.iter_mut().find(|r| {
                r["id"] == id && r["zoom"].as_f64() == Some(zoom) && r["budget_bytes"].as_u64() == Some(budget as u64)
            })
        {
            row["outcome"] = json!("running");
        }
    }
    pub(crate) fn completed(&self, mut report: Value) {
        if report.get("outcome").is_none() {
            report["outcome"] = json!(if report.get("error").is_some() {
                "failed"
            } else {
                "completed"
            });
        }
        if let Ok(mut s) = self.state.lock() {
            if let Some(row) = s.completed.iter_mut().find(|r| {
                r["id"] == report["id"]
                    && r["zoom"].as_f64() == report["zoom"].as_f64()
                    && r["budget_bytes"].as_u64() == report["budget_bytes"].as_u64()
            }) {
                for key in ["job", "display_name", "coverage", "input_bytes"] {
                    if report.get(key).is_none()
                        && let Some(value) = row.get(key)
                    {
                        report[key] = value.clone();
                    }
                }
                *row = report;
            } else {
                s.completed.push(report);
            }
        }
    }
    pub(crate) fn settle(&self, job: Option<&str>, cancelled: bool) {
        if let Ok(mut s) = self.state.lock() {
            for row in &mut s.completed {
                if job.is_some_and(|job| row["job"] != job) {
                    continue;
                }
                if row["outcome"] == "running" {
                    row["outcome"] = json!(if cancelled { "cancelled" } else { "failed" });
                } else if row["outcome"] == "pending" {
                    row["outcome"] = json!("skipped");
                }
            }
        }
    }
    /// The current case and stage, for a one-line progress notice.
    pub(crate) fn headline(&self) -> String {
        match self.state.lock() {
            Ok(s) if s.stage.is_empty() => s.case.clone(),
            Ok(s) => format!("{} · {}", s.case, s.stage),
            Err(_) => "Progress unavailable".into(),
        }
    }
    pub(crate) fn snapshot(&self) -> (String, Vec<Value>) {
        match self.state.lock() {
            Ok(s) => (
                format!(
                    "{} · {} · case {:.1}s · total {:.1}s · {} completed",
                    s.case,
                    s.stage,
                    s.case_started.elapsed().as_secs_f64(),
                    self.started.elapsed().as_secs_f64(),
                    s.completed.iter().filter(|r| r["outcome"] == "completed").count()
                ),
                s.completed.clone(),
            ),
            Err(_) => ("Progress unavailable".into(), vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn different_budgets_update_distinct_preplanned_rows() {
        let p = Progress::default();
        p.plan(crate::svg_lab::benchmarks::planned_cases(&[4.], &crate::svg_lab::benchmarks::COMPARISON_BUDGETS));
        p.running_budget("camera", 4., 32 * xgpui_svg::MIB);
        let rows = p.snapshot().1;
        assert_eq!(rows[0]["outcome"], "pending");
        assert_eq!(rows[1]["outcome"], "running");
        p.completed(json!({"id":"camera","zoom":4,"budget_bytes":32*xgpui_svg::MIB,"raster":{"median_ms":1.2}}));
        let rows = p.snapshot().1;
        assert_eq!(rows.len(), 8);
        assert_eq!(rows[0]["outcome"], "pending");
        assert_eq!(rows[1]["outcome"], "completed");
    }
    #[::core::prelude::v1::test]
    fn planned_rows_keep_order_through_execution_failure_and_cancellation() {
        let p = Progress::default();
        let plan = crate::benchmarks::svg_table::samples(1.);
        let ids: Vec<_> = plan.iter().map(|r| r["id"].clone()).collect();
        p.plan(plan);
        assert!(p.snapshot().1.iter().all(|r| r["outcome"] == "pending"));
        p.running("Sample 1", 1.);
        assert_eq!(p.snapshot().1[0]["outcome"], "running");
        p.completed(json!({"id":"Sample 1","zoom":1,"parse_ms":1.2}));
        p.running("Sample 2", 1.);
        p.completed(json!({"id":"Sample 2","zoom":1,"error":"bad artwork"}));
        p.running("Sample 3", 1.);
        p.settle(None, true);
        let rows = p.snapshot().1;
        assert_eq!(rows.iter().map(|r| r["id"].clone()).collect::<Vec<_>>(), ids);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0]["outcome"], "completed");
        assert_eq!(rows[1]["outcome"], "failed");
        assert_eq!(rows[2]["outcome"], "cancelled");
        assert_eq!(rows[3]["outcome"], "skipped");
        assert_eq!(rows[4]["outcome"], "skipped");
    }
    #[::core::prelude::v1::test]
    fn snapshots_report_current_stage_and_partial_cases_before_completion() {
        let p = Progress::default();
        let worker = p.clone();
        worker.case("Case 2 of 16 · grapes · 4×");
        worker.stage("Rasterizing sample 3 of 5");
        worker.completed(json!({"id":"first","zoom":1,"raster":{"median_ms":1.2}}));
        let (text, rows) = p.snapshot();
        assert!(text.contains("Case 2 of 16"));
        assert!(text.contains("sample 3 of 5"));
        assert!(text.contains("1 completed"));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["id"], "first");
        assert!(p.same(&worker));
        assert!(!p.same(&Progress::default()));
    }
}

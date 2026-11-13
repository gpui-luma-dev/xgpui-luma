//! Shared progress snapshots; worker stage messages travel on stderr, never files.
use super::*;
use std::sync::Mutex;
#[derive(Clone)]
pub(crate) struct Progress {
    state: Arc<Mutex<State>>,
    started: Instant,
    worker: bool,
    slot: Option<usize>,
}
#[derive(Clone)]
struct State {
    case: String,
    stage: String,
    case_started: Instant,
    completed: Vec<Value>,
    workers: std::collections::BTreeMap<usize, (String, String, Instant)>,
}
impl Default for Progress {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                case: "Preparing".into(),
                stage: String::new(),
                case_started: Instant::now(),
                completed: vec![],
                workers: Default::default(),
            })),
            started: Instant::now(),
            worker: false,
            slot: None,
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
    pub(crate) fn fork(&self, slot: usize) -> Self {
        Self { slot: Some(slot), ..self.clone() }
    }
    pub(crate) fn idle(&self) {
        if let Some(slot) = self.slot
            && let Ok(mut s) = self.state.lock()
        {
            s.workers.remove(&slot);
        }
    }
    pub(crate) fn case(&self, label: impl Into<String>) {
        if let Ok(mut s) = self.state.lock() {
            let label = label.into();
            if let Some(slot) = self.slot {
                s.workers.insert(slot, (label, "Starting".into(), Instant::now()));
                return;
            }
            s.case = label;
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
            if let Some(slot) = self.slot
                && let Some(worker) = s.workers.get_mut(&slot)
            {
                worker.1 = label;
            } else {
                s.stage = label;
            }
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
    pub(crate) fn completed(&self, mut report: Value) {
        if report.get("outcome").is_none() {
            report["outcome"] = json!(if report.get("error").is_some() {
                "failed"
            } else {
                "completed"
            });
        }
        if let Ok(mut s) = self.state.lock() {
            if let Some(row) = s
                .completed
                .iter_mut()
                .find(|r| r["id"] == report["id"] && r["zoom"].as_f64() == report["zoom"].as_f64())
            {
                if report.get("job").is_none()
                    && let Some(job) = row.get("job")
                {
                    report["job"] = job.clone();
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
    pub(crate) fn snapshot(&self) -> (String, Vec<Value>) {
        match self.state.lock() {
            Ok(s) => {
                let active = if s.workers.is_empty() {
                    format!("{} · {} · case {:.1}s", s.case, s.stage, s.case_started.elapsed().as_secs_f64())
                } else {
                    s.workers
                        .iter()
                        .map(|(index, (case, stage, started))| {
                            format!("Worker {}: {case} · {stage} · {:.1}s", index + 1, started.elapsed().as_secs_f64())
                        })
                        .collect::<Vec<_>>()
                        .join(" | ")
                };
                (
                    format!(
                        "{active} · total {:.1}s · {} completed",
                        self.started.elapsed().as_secs_f64(),
                        s.completed.iter().filter(|r| r["outcome"] == "completed").count()
                    ),
                    s.completed.clone(),
                )
            }
            Err(_) => ("Progress unavailable".into(), vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn parallel_worker_stages_do_not_overwrite_one_another() {
        let p = Progress::default();
        let first = p.fork(0);
        let second = p.fork(1);
        first.case("camera / 1×");
        second.case("grapes / 4×");
        first.stage("Parsing sample 2 of 5");
        second.stage("Rasterizing sample 1 of 5");
        let text = p.snapshot().0;
        assert!(text.contains("Worker 1: camera / 1× · Parsing sample 2 of 5"));
        assert!(text.contains("Worker 2: grapes / 4× · Rasterizing sample 1 of 5"));
        first.idle();
        let text = p.snapshot().0;
        assert!(!text.contains("Worker 1:"));
        assert!(text.contains("Worker 2:"));
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

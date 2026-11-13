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
    pub(crate) fn completed(&self, report: Value) {
        if let Ok(mut s) = self.state.lock() {
            s.completed.push(report);
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
                    s.completed.len()
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

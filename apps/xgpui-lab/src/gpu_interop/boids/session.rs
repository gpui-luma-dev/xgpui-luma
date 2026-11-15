//! UI-side ownership and measurements, matching the original capacity/pacing lab.
use super::engine::{Engine, Status};
use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    time::{Duration, Instant},
};
use crate::gpu_interop::Settings;

const LIVE_FRAMES: usize = 120;

pub(in crate::gpu_interop) struct Session {
    pub engine: Option<Engine>,
    pub starting: Option<gpui::Task<()>>,
    pub pause_check: Option<gpui::Task<()>>,
    pub error: Option<String>,
    pub ui_work: Rc<RefCell<VecDeque<f32>>>,
    pub measure_from: Option<Instant>,
    intervals: VecDeque<f32>,
    started: Instant,
    last_frame: Option<Instant>,
    last_presents: Option<u64>,
    applied_particles: u32,
    paused: bool,
    pub paused_baseline: Option<u64>,
    paused_steps: u64,
    paused_frames: u64,
    pauses: u64,
    frames: u64,
    repeated: u64,
    skipped: u64,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            engine: None,
            starting: None,
            pause_check: None,
            error: None,
            ui_work: Rc::default(),
            measure_from: None,
            intervals: VecDeque::new(),
            started: Instant::now(),
            last_frame: None,
            last_presents: None,
            applied_particles: super::boids::DEFAULT_PARTICLES,
            paused: false,
            paused_baseline: None,
            paused_steps: 0,
            paused_frames: 0,
            pauses: 0,
            frames: 0,
            repeated: 0,
            skipped: 0,
        }
    }
}
impl Session {
    pub fn install(&mut self, engine: Engine) {
        self.applied_particles = engine.status().particles;
        self.engine = Some(engine);
        self.error = None;
        self.reset_timings();
    }
    pub fn release(&mut self) {
        self.measure_from = None;
        self.starting = None;
        self.pause_check = None;
        if let Some(engine) = self.engine.take() {
            // Joining an own-thread simulation must not stall navigation.
            std::thread::spawn(move || drop(engine));
        }
    }
    fn reset_timings(&mut self) {
        self.started = Instant::now();
        self.last_frame = None;
        self.measure_from = None;
        self.last_presents = None;
        self.intervals.clear();
        self.ui_work.borrow_mut().clear();
        self.repeated = 0;
        self.skipped = 0;
    }
    pub fn apply(&mut self, settings: &Settings) {
        let Some(engine) = &mut self.engine else { return };
        let changed_thread = engine.threaded() != settings.threaded;
        if changed_thread {
            match engine.set_threaded(settings.threaded) {
                Ok(()) => self.error = None,
                Err(error) => self.error = Some(format!("Thread switch failed: {error:#}")),
            }
        }
        if settings.paused != self.paused {
            if !settings.paused
                && let Some(baseline) = self.paused_baseline.take()
            {
                self.paused_steps += engine.status().steps.saturating_sub(baseline);
            }
            if settings.paused {
                self.pauses += 1;
            }
            self.paused = settings.paused;
            self.last_frame = None;
            engine.set_paused(settings.paused);
        }
        if settings.particles != self.applied_particles {
            engine.set_particles(settings.particles);
            self.applied_particles = settings.particles;
        }
        engine.set_speed(settings.speed);
        engine.set_refresh_interval(settings.refresh);
        if changed_thread {
            self.reset_timings();
        }
    }
    pub fn frame(&mut self, now: Instant) -> (Option<Status>, bool) {
        let Some(engine) = &mut self.engine else {
            return (None, false);
        };
        let more = engine.step_inline();
        let status = engine.status();
        self.frames += 1;
        let warm = self.started.elapsed() >= Duration::from_millis(500);
        self.measure_from = (warm && !self.paused).then_some(now);
        if !warm {
            self.last_frame = None;
        }
        if self.paused {
            self.paused_frames += 1;
        } else if warm && let Some(previous) = self.last_frame.replace(now) {
            push_sample(&mut self.intervals, now.duration_since(previous).as_secs_f32() * 1000.);
        }
        let presents = engine.surface().stats().presents;
        if let Some(previous) = self.last_presents.replace(presents)
            && !self.paused
            && warm
        {
            match presents.saturating_sub(previous) {
                0 => self.repeated += 1,
                1 => {}
                _ => self.skipped += 1,
            }
        }
        (Some(status), more)
    }
    pub fn diagnostics(&self, status: &Status) -> Vec<(&'static str, String)> {
        let ui = self.ui_work.borrow();
        let surface = self.engine.as_ref().map(|engine| engine.surface().stats());
        let median = percentile(&self.intervals, 0.5);
        let hitches = self.intervals.iter().filter(|value| median > 0. && **value > median * 1.5).count();
        let paused_steps =
            self.paused_steps + self.paused_baseline.map_or(0, |baseline| status.steps.saturating_sub(baseline));
        let timing = |samples: &VecDeque<f32>| {
            format!(
                "p50 {:.2} / p95 {:.2} / max {:.2} ms",
                percentile(samples, 0.5),
                percentile(samples, 0.95),
                percentile(samples, 1.)
            )
        };
        vec![
            (
                "Engine",
                format!(
                    "{} · {}",
                    if status.threaded { "Own thread" } else { "UI thread" },
                    if status.paused { "paused" } else { "running" }
                ),
            ),
            ("Adapter", format!("{} · {}", status.adapter, status.format)),
            (
                "Capacity",
                format!(
                    "{} particles · {} steps · {:.1} steps/s",
                    status.particles, status.steps, status.steps_per_second
                ),
            ),
            ("Engine step CPU", format!("{:.2} ms (record + present)", status.record_ms)),
            (
                "Engine GPU",
                if !status.paused && status.threaded && status.gpu_ms > 0. {
                    format!("{:.2} ms", status.gpu_ms)
                } else {
                    "— (measured while running on own thread)".into()
                },
            ),
            ("UI work per frame", timing(&ui)),
            ("UI frame interval", timing(&self.intervals)),
            (
                "Pacing",
                format!(
                    "{} UI frames · {} hitches (>1.5× p50, last {LIVE_FRAMES} frames) · {} repeated · {} skipped",
                    self.frames, hitches, self.repeated, self.skipped
                ),
            ),
            (
                "Presentation",
                format!(
                    "{} · display {:.2} / refresh {:.2} ms · {} show timeouts · {} ring-full",
                    if status.observe_only {
                        "observe-only"
                    } else {
                        "GPU wait"
                    },
                    status.display_ms,
                    status.refresh_ms,
                    status.show_timeouts,
                    status.ring_full
                ),
            ),
            (
                "Compositor",
                surface.map_or_else(String::new, |s| {
                    format!(
                        "{} presents · {} composited · {} unshown · latency {:.2} ms",
                        s.presents, s.composited, s.unshown, s.composite_latency_ms
                    )
                }),
            ),
            (
                "Pause analysis",
                format!(
                    "{} pauses · {} UI frames while paused · {} engine steps after 100 ms settling",
                    self.pauses, self.paused_frames, paused_steps
                ),
            ),
        ]
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.release();
    }
}
pub(in crate::gpu_interop) fn push_sample(samples: &mut VecDeque<f32>, value: f32) {
    if samples.len() == LIVE_FRAMES {
        samples.pop_front();
    }
    samples.push_back(value);
}
fn percentile(samples: &VecDeque<f32>, q: f32) -> f32 {
    if samples.is_empty() {
        return 0.;
    }
    let mut sorted: Vec<_> = samples.iter().copied().collect();
    sorted.sort_by(f32::total_cmp);
    sorted[((sorted.len() - 1) as f32 * q) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires a compatible GPU, no GUI or window"]
    fn thread_comparison_preserves_flock_and_paused_engine_stops_steps() {
        let mut session = Session::default();
        session.install(Engine::new(500, true).unwrap());
        let mut settings = Settings { particles: 500, paused: true, ..Settings::default() };
        session.apply(&settings);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !session.engine.as_ref().unwrap().status().paused {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        let steps = session.engine.as_ref().unwrap().status().steps;
        settings.threaded = false;
        session.apply(&settings);
        assert!(!session.engine.as_ref().unwrap().threaded());
        assert!(session.engine.as_ref().unwrap().status().steps >= steps);
        settings.particles = 1500;
        settings.paused = false;
        session.apply(&settings);
        let (status, animate) = session.frame(Instant::now());
        let status = status.unwrap();
        assert!(animate);
        assert_eq!(status.particles, 1500);
        assert!(status.steps > steps);
        assert!(status.observe_only, "UI-thread mode never makes the compositor GPU-wait");
        assert!(status.error.is_none());
        settings.threaded = true;
        settings.paused = true;
        session.apply(&settings);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !session.engine.as_ref().unwrap().status().paused {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        std::thread::sleep(Duration::from_millis(100));
        let steps = session.engine.as_ref().unwrap().status().steps;
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(session.engine.as_ref().unwrap().status().steps, steps);
        assert!(session.engine.as_ref().unwrap().threaded());
        let failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let notify = failed.clone();
        let engine = session.engine.as_mut().unwrap();
        engine.on_failure(move || notify.store(true, std::sync::atomic::Ordering::Release));
        engine.surface().invalidate();
        engine.set_paused(false);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !failed.load(std::sync::atomic::Ordering::Acquire) {
            assert!(Instant::now() < deadline, "failure must notify the UI even without a new frame");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(engine.status().error.is_some());
        assert!(engine.status().paused);
    }
}

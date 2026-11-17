//! Background ownership of the upstream engines. No device setup or submission runs in render.
use super::{Kind, Settings, wave::WaveEngine, workbench};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use xgpui_gpu_interop::{EngineSurface, SurfaceDesc, SurfaceFormat};
use workbench::oracle::{Dab, HEIGHT, WIDTH};

pub(super) enum Command {
    Settings(Settings),
    Dab(Dab, Instant),
    Inspect(u32, u32),
    Reset,
    LoseDevice,
    Hold,
    Release,
}

#[derive(Clone, Default)]
pub(super) struct Snapshot {
    pub surface: Option<EngineSurface>,
    pub detail: String,
    pub inspection: String,
    pub inspection_data: Option<workbench::engine::Inspection>,
    pub revision: u64,
    pub error: Option<String>,
}

pub(super) struct Runtime {
    commands: mpsc::Sender<Command>,
    shared: Arc<Mutex<Snapshot>>,
}
impl Runtime {
    pub fn start(kind: Kind, settings: Settings) -> anyhow::Result<(Self, UnboundedReceiver<()>)> {
        let (commands, receiver) = mpsc::channel();
        let (wake, updates) = unbounded();
        let shared = Arc::new(Mutex::new(Snapshot::default()));
        let output = shared.clone();
        std::thread::Builder::new().name("lab-gpu-interop".into()).spawn(move || {
            let result = run(kind, settings, receiver, &output, &wake);
            if let Err(error) = result {
                output.lock().unwrap_or_else(|e| e.into_inner()).error = Some(format!("{error:#}"));
                let _ = wake.unbounded_send(());
            }
        })?;
        Ok((Self { commands, shared }, updates))
    }
    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }
    pub fn snapshot(&self) -> Snapshot {
        self.shared.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}
// Dropping the only sender disconnects the worker. It drops its engine and joins
// the upstream boids worker off the UI thread, including when navigation cancels setup.

enum Engine {
    Wave(Box<WaveEngine>),
    Workbench(Box<workbench::engine::Engine>),
}
impl Engine {
    fn new(kind: Kind, settings: &Settings) -> anyhow::Result<Self> {
        Ok(match kind {
            Kind::Waves => Self::Wave(Box::new(WaveEngine::new(
                SurfaceDesc {
                    width: WIDTH,
                    height: HEIGHT,
                    slots: 3,
                    format: if settings.wide_slots {
                        SurfaceFormat::Rgba16FloatLinearP3
                    } else {
                        SurfaceFormat::Bgra8Srgb
                    },
                },
                settings.work,
            )?)),
            Kind::Boids => anyhow::bail!("Boids is owned by its UI session"),
            Kind::Workbench => Self::Workbench(Box::new(workbench::engine::Engine::new(3)?)),
        })
    }
    fn surface(&self) -> &EngineSurface {
        match self {
            Self::Wave(e) => e.surface(),
            Self::Workbench(e) => e.surface(),
        }
    }
    fn settings(&mut self, settings: &Settings) {
        match self {
            Self::Wave(e) => {
                e.set_work(settings.work);
                e.set_bands(settings.bands);
            }
            Self::Workbench(e) => e.set_exposure(settings.exposure),
        }
    }
}

fn run(
    kind: Kind,
    mut settings: Settings,
    commands: mpsc::Receiver<Command>,
    output: &Mutex<Snapshot>,
    wake: &UnboundedSender<()>,
) -> anyhow::Result<()> {
    let mut engine = Engine::new(kind, &settings)?;
    engine.settings(&settings);
    let mut held = Vec::new();
    let mut snapshot = Snapshot::default();
    let mut lost_at: Option<Instant> = None;
    let mut rebuild_attempts = 0;
    let mut previous = None;
    let mut demo_step = 0_u64;
    let mut demo_point: Option<(f32, f32)> = None;
    let notify = wake.clone();
    engine.surface().on_present(move || {
        let _ = notify.unbounded_send(());
    });
    loop {
        // A bounded batch leaves room for publishing while a pointer is moving.
        let mut command = match commands.recv_timeout(Duration::from_millis(16)) {
            Ok(command) => Some(command),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
        };
        for batch in 0..256 {
            let Some(next) = command else { break };
            match next {
                Command::Settings(next) => {
                    engine.settings(&next);
                    settings = next;
                }
                Command::Dab(dab, sent) => {
                    if let Engine::Workbench(e) = &mut engine {
                        e.dab(dab);
                        e.note_edit_time(sent);
                    }
                }
                Command::Inspect(x, y) => {
                    if let Engine::Workbench(e) = &mut engine {
                        if !e.inspect(x, y) {
                            snapshot.inspection =
                                "Inspection unavailable: device lost or all four staging buffers are busy.".into();
                        }
                    }
                }
                Command::Reset => {
                    held.clear();
                    snapshot.inspection.clear();
                    snapshot.inspection_data = None;
                    demo_point = None;
                    if let Engine::Workbench(e) = &mut engine {
                        e.reset();
                    }
                }
                Command::LoseDevice => {
                    if let Engine::Workbench(e) = &mut engine {
                        e.lose_device();
                        lost_at = Some(Instant::now());
                        rebuild_attempts = 0;
                    }
                }
                Command::Hold => {
                    if held.len() < engine.surface().desc().slots
                        && let Some(content) = engine.surface().content()
                    {
                        held.push(content);
                    }
                }
                Command::Release => held.clear(),
            }
            command = if batch < 255 { commands.try_recv().ok() } else { None };
        }
        match &mut engine {
            Engine::Wave(e) => {
                if !settings.paused || e.surface().stats().presents == 0 {
                    e.frame()?;
                }
            }
            Engine::Workbench(e) => {
                if settings.demo && !e.is_lost() {
                    let t = demo_step as f32 * 0.05;
                    demo_step += 1;
                    let to =
                        (WIDTH as f32 * (0.5 + 0.42 * (t * 1.3).sin()), HEIGHT as f32 * (0.5 + 0.4 * (t * 2.1).cos()));
                    let from = demo_point.replace(to).unwrap_or(to);
                    let steps = (((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt()
                        / (settings.radius / 3.).max(1.))
                    .ceil()
                    .max(1.) as u32;
                    for i in 1..=steps {
                        let fraction = i as f32 / steps as f32;
                        e.dab(Dab {
                            stroke_id: 0x8000_0000 | ((demo_step / 120) as u32 & 0x7fff_ffff),
                            x: from.0 + (to.0 - from.0) * fraction,
                            y: from.1 + (to.1 - from.1) * fraction,
                            radius: settings.radius,
                            opacity: settings.opacity,
                            color: super::COLORS[(demo_step / 120) as usize % super::COLORS.len()],
                        });
                    }
                } else {
                    demo_point = None;
                }
                if e.lost_by_driver() {
                    e.mark_lost();
                    lost_at = Some(Instant::now());
                }
                if e.is_lost() {
                    if lost_at.is_some_and(|at| at.elapsed() > Duration::from_millis(500)) {
                        match e.rebuild() {
                            Ok(()) => {
                                lost_at = None;
                                snapshot.error = None;
                                let notify = wake.clone();
                                e.surface().on_present(move || {
                                    let _ = notify.unbounded_send(());
                                });
                            }
                            Err(error) => {
                                snapshot.error = Some(format!("Rebuild failed: {error:#}"));
                                rebuild_attempts += 1;
                                lost_at = (rebuild_attempts < 20).then(Instant::now);
                            }
                        }
                    }
                } else {
                    e.frame()?;
                    // Keep the live lab bounded over long editing sessions.
                    for samples in [&mut e.latencies_ms, &mut e.record_ms, &mut e.present_ms]
                        .into_iter()
                        .chain(e.phase_ms.iter_mut())
                    {
                        if samples.len() > 1200 {
                            samples.drain(..samples.len() - 1200);
                        }
                    }
                    if let Some(found) = e.poll_inspections().pop() {
                        snapshot.inspection_data = Some(found.clone());
                        snapshot.inspection = format!(
                            "4×4 at ({}, {}) · revision {} · generation {} · first linear P3 pixel {:?}",
                            found.x,
                            found.y,
                            found.revision,
                            found.generation,
                            found.pixels.first()
                        );
                    }
                }
                let s = e.stats();
                snapshot.revision = s.revision;
                snapshot.detail = format!(
                    "{} · revision {} / published {} · {} dabs · {} paint passes · {} touched tiles · {} publishes · {} blocked · {} submissions · {} objects · {} B GPU published / {} B uploaded / {} B read · {} inspections · generation {} · {} recoveries · {} replayed in {:.2} ms · {} pending · {} inspections pending · exposure {:+.1} · {} · {}",
                    e.adapter(),
                    s.revision,
                    s.published_revision,
                    s.dabs_applied,
                    s.paint_passes,
                    s.tiles_touched,
                    s.publishes,
                    s.publish_blocked,
                    s.submissions,
                    s.resources_created,
                    s.gpu_bytes_published,
                    s.cpu_bytes_uploaded,
                    s.cpu_bytes_read,
                    s.inspections,
                    e.generation(),
                    s.recoveries,
                    s.replayed_dabs,
                    s.replay_ms,
                    e.pending_dabs(),
                    e.inspections_in_flight(),
                    e.exposure(),
                    if e.gpu_sync() { "GPU sync" } else { "CPU sync" },
                    if e.busy() { "busy" } else { "idle" }
                );
            }
        }
        snapshot.surface = Some(engine.surface().clone());
        if kind == Kind::Waves {
            snapshot.detail = format!(
                "Independent wgpu device · {} work iterations · {} submission bands",
                settings.work, settings.bands
            );
        }
        if !held.is_empty() {
            snapshot.detail.push_str(&format!(" · {} held snapshots", held.len()));
        }
        if let Some(error) = engine.surface().unsupported() {
            snapshot.error = Some(error.reason().to_owned());
        }
        let signature = (
            snapshot.detail.clone(),
            snapshot.inspection.clone(),
            snapshot.error.clone(),
            engine.surface().stats().presents,
        );
        *output.lock().unwrap_or_else(|e| e.into_inner()) = snapshot.clone();
        if previous.as_ref() != Some(&signature) {
            previous = Some(signature);
            let _ = wake.unbounded_send(());
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
impl Runtime {
    pub(super) fn recording() -> (Self, mpsc::Receiver<Command>) {
        let (commands, receiver) = mpsc::channel();
        (Self { commands, shared: Arc::new(Mutex::new(Snapshot::default())) }, receiver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires a compatible GPU, no GUI or window"]
    fn workbench_worker_applies_commands_and_delivers_inspection_updates() {
        let (runtime, _updates) = Runtime::start(Kind::Workbench, Settings::default()).unwrap();
        let wait = |ready: &dyn Fn(&Snapshot) -> bool| {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let snapshot = runtime.snapshot();
                assert!(snapshot.error.is_none(), "{:?}", snapshot.error);
                if ready(&snapshot) {
                    break;
                }
                assert!(Instant::now() < deadline, "worker did not publish expected update");
                std::thread::sleep(Duration::from_millis(10));
            }
        };
        wait(&|snapshot| snapshot.surface.as_ref().is_some_and(|surface| surface.stats().presents > 0));
        runtime.send(Command::Dab(
            Dab { stroke_id: 0, x: 100., y: 100., radius: 10., opacity: 1., color: [1., 0., 0.] },
            Instant::now(),
        ));
        runtime.send(Command::Inspect(100, 100));
        wait(&|snapshot| snapshot.inspection.contains("revision 2"));
        let inspected = runtime.snapshot().inspection_data.unwrap();
        assert_eq!((inspected.x, inspected.y, inspected.revision), (100, 100, 2));
        assert_eq!(inspected.pixels.len(), 16);
        runtime.send(Command::Reset);
        wait(&|snapshot| {
            snapshot.inspection.is_empty()
                && snapshot.inspection_data.is_none()
                && snapshot.detail.contains("revision 3 / published 3")
        });
    }

    #[test]
    #[ignore = "requires a compatible GPU, no GUI or window"]
    fn independent_wave_and_boids_engines_publish_frames() {
        let settings = Settings::default();
        let mut wave = Engine::new(Kind::Waves, &settings).unwrap();
        let Engine::Wave(engine) = &mut wave else {
            unreachable!()
        };
        assert!(engine.frame().unwrap());
        assert_eq!(engine.surface().stats().presents, 1);
        assert!(engine.surface().content().is_some());
        let boids = super::super::boids::engine::Engine::new(settings.particles, true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while (boids.surface().stats().presents == 0 || boids.surface().content().is_none())
            && Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(boids.surface().stats().presents > 0);
        assert!(boids.surface().content().is_some());
    }
}

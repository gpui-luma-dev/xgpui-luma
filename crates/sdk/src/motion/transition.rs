//! Pluggable scalar motion. Definitions are shared configuration; running motions
//! sample absolute elapsed time without a window, scheduler, or control dependency.

use std::{fmt::Debug, sync::Arc, time::Duration};

/// Position and velocity (units per second), retained across interruptions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MotionState {
    pub value: f32,
    pub velocity: f32,
}

/// A sample and the implementation's completion decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionSample {
    pub state: MotionState,
    pub complete: bool,
}

/// Creates an independent trajectory from the current position and velocity.
pub trait MotionDefinition: Debug + Send + Sync {
    fn start(&self, current: MotionState, target: f32) -> Box<dyn RunningMotion>;
    /// Adjust a changing measured destination within one interaction. Timed
    /// definitions retain the original deadline; physics retains sampled velocity.
    fn retarget(&self, current: MotionState, target: f32, _elapsed: Duration) -> Box<dyn RunningMotion> {
        self.start(current, target)
    }
}

/// A deterministic trajectory. `elapsed` is absolute time since `start`, not a
/// frame delta. Sampling must not mutate state, so cloned controls can share it.
/// Completion must return the exact target with zero velocity. Values may
/// overshoot; templates clamp bounded properties such as opacity separately.
pub trait RunningMotion: Debug + Send + Sync {
    fn sample(&self, elapsed: Duration) -> MotionSample;
}

/// Look-owned transition configuration. Controls execute it through
/// [`super::VisualTransition`] without selecting an animation implementation.
///
/// ```no_run
/// use xgpui_luma::{controls::{switch, tabs::Tabs}, motion::Transition};
///
/// let switch = switch::new("notifications")
///     .motion(|m| m.state = Transition::spring(300.0, 28.0));
/// let tabs = Tabs::new("settings")
///     .motion(|m| {
///         m.indicator = Transition::spring(300.0, 28.0);
///         m.content = Transition::instant();
///     });
/// ```
#[derive(Clone, Debug)]
pub struct Transition {
    definition: Arc<dyn MotionDefinition>,
    key: Option<ConfigKey>,
}

/// Configuration for diagnostics; custom definitions retain their own `Debug`
/// representation. Controls never use this to choose an animation engine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransitionDescription {
    Timed { duration: Duration, easing: Easing },
    Spring { stiffness: f32, damping: f32 },
    Instant,
    Custom,
}

// Configuration equality prevents render-time reconstruction of the same look
// value from cancelling active trajectories. This never selects an engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConfigKey {
    Timed(Duration, Easing),
    Spring(u32, u32),
}

impl Default for Transition {
    fn default() -> Self {
        crate::theme::MotionDefaults::vanilla().transition.clone()
    }
}

impl Transition {
    /// Inspect the authored configuration without assuming a duration exists.
    pub fn description(&self) -> TransitionDescription {
        match self.key {
            Some(ConfigKey::Timed(duration, _)) if duration.is_zero() => TransitionDescription::Instant,
            Some(ConfigKey::Timed(duration, easing)) => TransitionDescription::Timed { duration, easing },
            Some(ConfigKey::Spring(stiffness, damping)) => {
                TransitionDescription::Spring { stiffness: f32::from_bits(stiffness), damping: f32::from_bits(damping) }
            }
            None => TransitionDescription::Custom,
        }
    }

    pub(crate) fn same_definition(&self, other: &Self) -> bool {
        (self.key.is_some() && self.key == other.key) || Arc::ptr_eq(&self.definition, &other.definition)
    }
    pub fn timed(duration: Duration, easing: Easing) -> Self {
        Self { definition: Arc::new(Timed { duration, easing }), key: Some(ConfigKey::Timed(duration, easing)) }
    }

    /// Unit-mass spring. Non-finite or non-positive parameters settle immediately.
    pub fn spring(stiffness: f32, damping: f32) -> Self {
        if !stiffness.is_finite() || !damping.is_finite() || stiffness <= 0.0 || damping <= 0.0 {
            return Self::instant();
        }
        Self {
            definition: Arc::new(Spring { stiffness: stiffness.into(), damping: damping.into() }),
            key: Some(ConfigKey::Spring(stiffness.to_bits(), damping.to_bits())),
        }
    }

    pub fn instant() -> Self {
        Self::timed(Duration::ZERO, Easing::Linear)
    }

    pub fn custom(definition: impl MotionDefinition + 'static) -> Self {
        Self { definition: Arc::new(definition), key: None }
    }

    pub fn retarget(&self, current: MotionState, target: f32, elapsed: Duration) -> Box<dyn RunningMotion> {
        self.definition.retarget(current, target, elapsed)
    }

    pub fn start(&self, current: MotionState, target: f32) -> Box<dyn RunningMotion> {
        self.definition.start(current, target)
    }
}

/// Existing cubic behavior plus linear interpolation. Further curves can be
/// supplied through a custom definition without changing controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Easing {
    Linear,
    #[default]
    OutCubic,
}

#[derive(Debug)]
struct Timed {
    duration: Duration,
    easing: Easing,
}

#[derive(Debug)]
struct TimedRun {
    from: f32,
    target: f32,
    duration: Duration,
    easing: Easing,
}

impl MotionDefinition for Timed {
    fn retarget(&self, current: MotionState, target: f32, elapsed: Duration) -> Box<dyn RunningMotion> {
        Timed { duration: self.duration.saturating_sub(elapsed), easing: self.easing }.start(current, target)
    }
    fn start(&self, current: MotionState, target: f32) -> Box<dyn RunningMotion> {
        Box::new(TimedRun { from: current.value, target, duration: self.duration, easing: self.easing })
    }
}

fn settled(target: f32) -> MotionSample {
    MotionSample { state: MotionState { value: target, velocity: 0.0 }, complete: true }
}

impl RunningMotion for TimedRun {
    fn sample(&self, elapsed: Duration) -> MotionSample {
        if elapsed >= self.duration {
            return settled(self.target);
        }
        let t = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let (factor, derivative) = match self.easing {
            Easing::Linear => (t, 1.0),
            Easing::OutCubic => (super::ease_out_cubic(t), 3.0 * (1.0 - t).powi(2)),
        };
        let distance = self.target - self.from;
        MotionSample {
            state: MotionState {
                value: self.from + distance * factor,
                velocity: distance * derivative / self.duration.as_secs_f32(),
            },
            complete: false,
        }
    }
}

#[derive(Debug)]
struct Spring {
    stiffness: f64,
    damping: f64,
}

#[derive(Debug)]
struct SpringRun {
    stiffness: f64,
    damping: f64,
    current: MotionState,
    target: f32,
}

impl MotionDefinition for Spring {
    fn start(&self, current: MotionState, target: f32) -> Box<dyn RunningMotion> {
        Box::new(SpringRun { stiffness: self.stiffness, damping: self.damping, current, target })
    }
}

impl RunningMotion for SpringRun {
    fn sample(&self, elapsed: Duration) -> MotionSample {
        if self.current.value == self.target && self.current.velocity == 0.0 {
            return settled(self.target);
        }
        if elapsed.is_zero() {
            return MotionSample { state: self.current, complete: false };
        }
        let t = elapsed.as_secs_f64();
        let x = f64::from(self.current.value) - f64::from(self.target);
        let v = f64::from(self.current.velocity);
        let a = self.damping / 2.0;
        let discriminant = a * a - self.stiffness;
        let (offset, velocity) = if discriminant.abs() <= self.stiffness * 1e-6 {
            let b = v + a * x;
            let decay = (-a * t).exp();
            ((x + b * t) * decay, (b - a * (x + b * t)) * decay)
        } else if discriminant < 0.0 {
            let w = (-discriminant).sqrt();
            let b = (v + a * x) / w;
            let (sin, cos) = (w * t).sin_cos();
            let displacement = x * cos + b * sin;
            let decay = (-a * t).exp();
            (displacement * decay, (-a * displacement - x * w * sin + b * w * cos) * decay)
        } else {
            let root = discriminant.sqrt();
            let r1 = -self.stiffness / (a + root);
            let r2 = -a - root;
            let c1 = (v - r2 * x) / (r1 - r2);
            let c2 = x - c1;
            let p = c1 * (r1 * t).exp();
            let q = c2 * (r2 * t).exp();
            (p + q, r1 * p + r2 * q)
        };
        if offset.abs() < 0.0001 && velocity.abs() < 0.001 {
            return settled(self.target);
        }
        MotionSample {
            state: MotionState { value: (f64::from(self.target) + offset) as f32, velocity: velocity as f32 },
            complete: false,
        }
    }
}

type MotionEdit<M> = Arc<dyn Fn(&mut M) + Send + Sync>;

/// Deferred SDK-owned edits to an inherited, control-specific profile. Look
/// builders forward this value; they do not implement override resolution.
#[derive(Clone)]
pub struct MotionOverrides<M>(Vec<MotionEdit<M>>);

impl<M> Default for MotionOverrides<M> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<M> MotionOverrides<M> {
    pub fn push(&mut self, configure: impl Fn(&mut M) + Send + Sync + 'static) {
        self.0.push(Arc::new(configure));
    }

    pub fn append(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    pub fn apply(&self, profile: &mut M) {
        for configure in &self.0 {
            configure(profile);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_describe_configuration_without_a_fake_spring_duration() {
        assert_eq!(
            Transition::default().description(),
            TransitionDescription::Timed { duration: Duration::from_millis(200), easing: Easing::OutCubic }
        );
        assert_eq!(
            Transition::spring(300.0, 28.0).description(),
            TransitionDescription::Spring { stiffness: 300.0, damping: 28.0 }
        );
        assert_eq!(Transition::instant().description(), TransitionDescription::Instant);
        assert_eq!(Transition::custom(Custom).description(), TransitionDescription::Custom);
    }

    #[test]
    fn geometry_retarget_retains_timed_deadline_and_spring_velocity() {
        let timed = Transition::timed(Duration::from_millis(200), Easing::Linear);
        let current = MotionState { value: 80.0, velocity: 400.0 };
        let corrected = timed.retarget(current, 120.0, Duration::from_millis(150));
        assert!(!corrected.sample(Duration::from_millis(25)).complete);
        assert_eq!(corrected.sample(Duration::from_millis(50)), settled(120.0));
        let spring = Transition::spring(300.0, 28.0).retarget(current, -20.0, Duration::from_secs(2));
        assert_eq!(spring.sample(Duration::ZERO).state, current);
        assert_eq!(spring.sample(Duration::from_secs(10)), settled(-20.0));
    }

    #[test]
    fn timed_samples_preserve_existing_curve_and_completion() {
        let run = Transition::timed(Duration::from_secs(1), Easing::OutCubic).start(MotionState::default(), 1.0);
        let halfway = run.sample(Duration::from_millis(500));
        assert_eq!(halfway.state.value, 0.875);
        assert_eq!(halfway.state.velocity, 0.75);
        assert!(!halfway.complete);
        assert_eq!(run.sample(Duration::from_secs(1)), settled(1.0));
        assert_eq!(run.sample(Duration::from_secs(20)), settled(1.0));
    }

    #[test]
    fn springs_cover_all_damping_regimes_and_settle_exactly() {
        for damping in [10.0, 20.0, 40.0] {
            let run = Transition::spring(100.0, damping).start(MotionState::default(), 1.0);
            assert_eq!(run.sample(Duration::ZERO).state, MotionState::default());
            let sample = run.sample(Duration::from_millis(100));
            assert!(sample.state.value > 0.0);
            assert!(sample.state.velocity > 0.0);
            assert!(!sample.complete);
            assert_eq!(run.sample(Duration::from_secs(20)), settled(1.0));
        }
    }

    #[test]
    fn spring_retarget_preserves_position_and_velocity_in_pixel_units() {
        let transition = Transition::spring(200.0, 18.0);
        let run = transition.start(MotionState { value: 40.0, velocity: 0.0 }, 280.0);
        let interrupted = run.sample(Duration::from_millis(100)).state;
        let reversed = transition.start(interrupted, -120.0);
        let initial = reversed.sample(Duration::ZERO).state;
        assert!((initial.value - interrupted.value).abs() < 0.0001);
        assert!((initial.velocity - interrupted.velocity).abs() < 0.0001);
        assert_eq!(reversed.sample(Duration::from_secs(10)), settled(-120.0));
    }

    #[test]
    fn spring_can_overshoot_and_invalid_parameters_are_immediate() {
        let run = Transition::spring(100.0, 5.0).start(MotionState::default(), 1.0);
        assert!(run.sample(Duration::from_millis(350)).state.value > 1.0);
        for (stiffness, damping) in [(0.0, 10.0), (100.0, 0.0), (f32::NAN, 1.0), (1.0, f32::INFINITY)] {
            let run = Transition::spring(stiffness, damping).start(MotionState::default(), 1.0);
            assert_eq!(run.sample(Duration::ZERO), settled(1.0));
        }
    }

    #[derive(Debug)]
    struct Custom;
    #[derive(Debug)]
    struct CustomRun(f32);
    impl MotionDefinition for Custom {
        fn start(&self, _: MotionState, target: f32) -> Box<dyn RunningMotion> {
            Box::new(CustomRun(target))
        }
    }
    impl RunningMotion for CustomRun {
        fn sample(&self, _: Duration) -> MotionSample {
            settled(self.0)
        }
    }

    #[test]
    fn custom_motion_runs_through_the_existing_channel_without_engine_selection() {
        let mut channel = crate::motion::VisualTransition::with_transition(5.0, Transition::custom(Custom));
        channel.set_target(200.0);
        assert_eq!(channel.progress(), 200.0);
        assert!(!channel.is_animating());
    }
}

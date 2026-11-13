//! Upload stress model adapted from ../xgpui/apps/xgpui-ext-lab/media-lab/src/model.rs
//! at 778c123. Keep the source's frame policy, slot states and preview retirement.
use std::path::Path;
use xgpui_media::{
    FrameUploadLimits, ImageJob, LoadStatus, MediaImage, MediaLimits, MediaLoader, UploadLimits, UploadStatus,
};

pub(super) const BURST_SIZE: usize = 12;
pub(super) const COUNTS: [usize; 3] = [1, 2, 4];
pub(super) const BYTES: [usize; 3] = [16 * super::MIB, 32 * super::MIB, 64 * super::MIB];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Waiting,
    Decoding,
    Staging,
    Ready,
    Retired,
    Failed,
}
impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Waiting => "WAIT",
            Self::Decoding => "DECODE",
            Self::Staging => "STAGE",
            Self::Ready => "READY",
            Self::Retired => "DONE",
            Self::Failed => "ERROR",
        }
    }
}
pub(super) struct Slot {
    pub job: Option<ImageJob>,
    pub image: Option<MediaImage>,
    pub phase: Phase,
    pub error: Option<String>,
}
impl Slot {
    pub fn waiting() -> Self {
        Self { job: None, image: None, phase: Phase::Waiting, error: None }
    }
    fn poll(&mut self) {
        if let Some(job) = self.job.as_mut() {
            if let Some(image) = job.try_take() {
                self.image = Some(image);
                self.phase = Phase::Staging;
                self.job = None;
            } else if let LoadStatus::Failed(error) = job.status() {
                self.phase = Phase::Failed;
                self.error = Some(error);
                self.job = None;
            }
        }
        if let Some(image) = &self.image {
            self.phase = match image.upload_status() {
                UploadStatus::Ready => Phase::Ready,
                UploadStatus::Failed(error) => {
                    self.error = Some(error);
                    Phase::Failed
                }
                _ => Phase::Staging,
            };
        }
    }
    pub fn pending(&self) -> bool {
        matches!(self.phase, Phase::Waiting | Phase::Decoding | Phase::Staging)
    }
    fn admit(&mut self, loader: &MediaLoader, path: &Path) -> anyhow::Result<()> {
        if self.phase == Phase::Waiting {
            self.job = Some(loader.load(path)?);
            self.phase = Phase::Decoding;
        }
        Ok(())
    }
}

pub(super) struct ResourceLab {
    pub loader: anyhow::Result<MediaLoader>,
    pub primary: Slot,
    pub burst: Vec<Slot>,
    pub count_limit: usize,
    pub byte_limit: usize,
    pub admission: String,
    pub settle_frames: u8,
}
impl ResourceLab {
    pub fn new() -> Self {
        Self {
            loader: Self::loader(COUNTS[0], BYTES[0]),
            primary: Slot::waiting(),
            burst: Vec::new(),
            count_limit: COUNTS[0],
            byte_limit: BYTES[0],
            admission: String::new(),
            settle_frames: 0,
        }
    }
    fn loader(count: usize, bytes: usize) -> anyhow::Result<MediaLoader> {
        MediaLoader::with_upload_policy(
            MediaLimits::default(),
            UploadLimits::default(),
            FrameUploadLimits { max_uploads: count, max_bytes: bytes },
        )
    }
    pub fn reload(&mut self) {
        self.burst.clear();
        self.primary = Slot::waiting();
        self.admission.clear();
        self.settle_frames = 0;
    }
    pub fn start_burst(&mut self) {
        self.burst = (0..BURST_SIZE).map(|_| Slot::waiting()).collect();
    }
    pub fn policy(&mut self, count: usize, bytes: usize) {
        self.loader = Self::loader(count, bytes);
        self.count_limit = count;
        self.byte_limit = bytes;
        self.reload();
    }
    pub fn submitted(&self) -> usize {
        self.burst.iter().filter(|slot| matches!(slot.phase, Phase::Ready | Phase::Retired)).count()
    }
    pub fn pending(&self) -> bool {
        self.primary.pending() || self.burst.iter().any(Slot::pending)
    }
    pub fn needs_redraw(&self) -> bool {
        self.loader.is_ok() && (self.pending() || self.settle_frames > 0)
    }
    pub fn error(&self) -> Option<String> {
        self.loader
            .as_ref()
            .err()
            .map(ToString::to_string)
            .or_else(|| self.primary.error.clone())
            .or_else(|| self.burst.iter().find_map(|slot| slot.error.clone()))
    }
    fn retire_previews(&mut self) {
        // Match the original: keep the newest ready preview while waiting work
        // needs capacity. Terminal slot labels remain after older sources retire.
        if self.burst.iter().any(|slot| slot.phase == Phase::Waiting) {
            let ready: Vec<usize> = self
                .burst
                .iter()
                .enumerate()
                .filter(|(_, slot)| slot.phase == Phase::Ready)
                .map(|(index, _)| index)
                .collect();
            for &index in ready.iter().take(ready.len().saturating_sub(1)) {
                self.burst[index].image = None;
                self.burst[index].phase = Phase::Retired;
            }
        }
    }
    pub fn poll(&mut self, path: &Path) {
        self.admission.clear();
        self.primary.poll();
        if let Ok(loader) = &self.loader
            && let Err(error) = self.primary.admit(loader, path)
        {
            self.admission = error.to_string();
        }
        for slot in &mut self.burst {
            slot.poll();
        }
        self.retire_previews();
        if let Ok(loader) = &self.loader {
            for slot in &mut self.burst {
                if let Err(error) = slot.admit(loader, path) {
                    self.admission = error.to_string();
                    break;
                }
            }
        }
        self.settle_frames = if self.pending() {
            6
        } else {
            self.settle_frames.saturating_sub(1)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn fixture(side: u32) -> (tempfile::TempDir, std::path::PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sample.png");
        image::RgbaImage::from_pixel(side, side, image::Rgba([32, 128, 224, 255])).save(&path).unwrap();
        (directory, path)
    }
    fn decode(loader: &MediaLoader, path: &Path) -> MediaImage {
        let mut job = loader.load(path).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(image) = job.try_take() {
                return image;
            }
            assert!(!matches!(job.status(), LoadStatus::Failed(_)), "{:?}", job.status());
            assert!(Instant::now() < deadline, "decode timed out");
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn deferred_primary_retries_and_policy_reload_clears_old_work() {
        let (_directory, path) = fixture(8);
        let mut lab = ResourceLab::new();
        let loader = lab.loader.as_ref().unwrap().clone();
        // The original library reserves its per-image maximum, independent of the
        // tiny fixture's actual payload: four snapshots exhaust the default pool.
        let mut held: Vec<_> = (0..4).map(|_| decode(&loader, &path)).collect();
        lab.poll(&path);
        assert_eq!(lab.primary.phase, Phase::Waiting);
        assert!(!lab.admission.is_empty());
        assert!(lab.primary.error.is_none(), "backpressure is retried, not a failed image");
        held.pop();
        lab.poll(&path);
        assert_eq!(lab.primary.phase, Phase::Decoding);
        lab.start_burst();
        assert_eq!(lab.burst.len(), BURST_SIZE);
        lab.policy(4, BYTES[2]);
        assert!(lab.burst.is_empty(), "policy changes clear the run, matching upstream");
        assert_eq!(lab.primary.phase, Phase::Waiting);
        assert!(lab.primary.job.is_none());
        assert_eq!(lab.count_limit, 4);
        assert_eq!(lab.byte_limit, BYTES[2]);
    }

    #[cfg(target_os = "macos")]
    mod native {
        use super::*;
        use gpui::{
            Bounds, ContentMask, CustomDraw, DevicePixels, DrawOrder, PlatformHeadlessRenderer, ScaledPixels, Scene,
            point, size,
        };
        use std::sync::Arc;

        fn scene(images: &[MediaImage]) -> Scene {
            let mut scene = Scene::default();
            for (index, image) in images.iter().enumerate() {
                let bounds = Bounds::new(
                    point(ScaledPixels(index as f32 * 8.), ScaledPixels(0.)),
                    size(ScaledPixels(8.), ScaledPixels(8.)),
                );
                scene.insert_primitive(CustomDraw {
                    order: DrawOrder::default(),
                    bounds,
                    content_mask: ContentMask { bounds },
                    corner_radii: Default::default(),
                    opacity: 1.,
                    content: Arc::new(image.clone()),
                    coverage: None,
                });
            }
            scene.finish();
            scene
        }
        fn stage(renderer: &mut dyn PlatformHeadlessRenderer, image: &MediaImage) -> anyhow::Result<()> {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                match image.upload_status() {
                    UploadStatus::Staged => return Ok(()),
                    UploadStatus::Idle | UploadStatus::Queued => {
                        renderer.render_scene_to_image(
                            &scene(std::slice::from_ref(image)),
                            size(DevicePixels(128), DevicePixels(16)),
                        )?;
                    }
                    UploadStatus::Failed(error) => anyhow::bail!(error),
                    _ => {}
                }
                anyhow::ensure!(Instant::now() < deadline, "staging timed out");
                std::thread::sleep(Duration::from_millis(2));
            }
        }

        #[test]
        fn native_resource_policy_limits_count_and_bytes_and_completes_twelve_slots() -> anyhow::Result<()> {
            // Offscreen renderer only: this test never opens an application window.
            let mut renderer = gpui_apple::metal_renderer::MetalHeadlessRenderer::new()?;
            let (_directory, path) = fixture(1024);
            for (count, bytes, expected) in [(1, BYTES[2], 1), (4, BYTES[0], 2), (4, BYTES[1], 3)] {
                let loader = ResourceLab::loader(count, bytes)?;
                let mut images = Vec::new();
                for _ in 0..3 {
                    let image = decode(&loader, &path);
                    stage(&mut renderer, &image)?;
                    images.push(image);
                }
                renderer.render_scene_to_image(&scene(&images), size(DevicePixels(128), DevicePixels(16)))?;
                assert_eq!(
                    images.iter().filter(|image| image.upload_status() == UploadStatus::Ready).count(),
                    expected,
                    "count {count}, bytes {bytes}: actual per-frame submissions"
                );
            }
            let (_directory, path) = fixture(64);
            let mut lab = ResourceLab::new();
            lab.start_burst();
            let deadline = Instant::now() + Duration::from_secs(20);
            while lab.pending() || lab.settle_frames > 0 {
                lab.poll(&path);
                anyhow::ensure!(lab.error().is_none(), "{:?}", lab.error());
                let images: Vec<_> = lab
                    .primary
                    .image
                    .iter()
                    .chain(lab.burst.iter().filter_map(|slot| slot.image.as_ref()))
                    .cloned()
                    .collect();
                let before = images.iter().filter(|image| image.upload_status() == UploadStatus::Ready).count();
                renderer.render_scene_to_image(&scene(&images), size(DevicePixels(128), DevicePixels(16)))?;
                let after = images.iter().filter(|image| image.upload_status() == UploadStatus::Ready).count();
                assert!(after - before <= lab.count_limit);
                anyhow::ensure!(
                    Instant::now() < deadline,
                    "burst timed out: {} submitted, admission {}",
                    lab.submitted(),
                    lab.admission
                );
                std::thread::sleep(Duration::from_millis(2));
            }
            assert_eq!(lab.submitted(), BURST_SIZE);
            assert_eq!(lab.burst.len(), BURST_SIZE, "all status chips survive completion");
            assert!(lab.burst.iter().any(|slot| slot.phase == Phase::Retired));
            assert!(
                lab.burst.iter().any(|slot| slot.phase == Phase::Ready && slot.image.is_some()),
                "final previews stay visible, as in upstream"
            );
            Ok(())
        }
    }
}

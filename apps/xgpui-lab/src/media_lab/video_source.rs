//! Native source and shared moving fixture synchronized from xgpui-ext-lab at 74b8e4e.
#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
mod pattern;
#[cfg(target_os = "windows")]
pub(crate) mod windows;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use anyhow::Result;
#[cfg(any(target_os = "macos", target_os = "windows"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceFormat {
    #[default]
    Bgra,
    #[cfg(target_os = "macos")]
    Full601,
    #[cfg(target_os = "macos")]
    Video601,
    #[cfg(target_os = "macos")]
    Full709,
    #[cfg(target_os = "macos")]
    Video709,
}
#[cfg(any(target_os = "macos", target_os = "windows"))]
impl SourceFormat {
    pub(super) const ALL: &'static [Self] = &[
        Self::Bgra,
        #[cfg(target_os = "macos")]
        Self::Full601,
        #[cfg(target_os = "macos")]
        Self::Video601,
        #[cfg(target_os = "macos")]
        Self::Full709,
        #[cfg(target_os = "macos")]
        Self::Video709,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Bgra => "BGRA8",
            #[cfg(target_os = "macos")]
            Self::Full601 => "NV12 opaque · full · BT.601",
            #[cfg(target_os = "macos")]
            Self::Video601 => "NV12 opaque · video · BT.601",
            #[cfg(target_os = "macos")]
            Self::Full709 => "NV12 opaque · full · BT.709",
            #[cfg(target_os = "macos")]
            Self::Video709 => "NV12 opaque · video · BT.709",
        }
    }
    #[cfg(target_os = "macos")]
    pub(super) fn frame(self, index: u64) -> Result<xgpui_media::CoreVideoFrame> {
        macos::frame(self, index)
    }
    fn video_frame(self, index: u64) -> Result<xgpui_media::VideoFrame> {
        let timestamp = std::time::Duration::from_micros(index * 33_333);
        #[cfg(target_os = "macos")]
        {
            Ok(xgpui_media::VideoFrame::core_video(self.frame(index)?, timestamp))
        }
        #[cfg(target_os = "windows")]
        {
            windows::Snapshot::new(index).video_frame(timestamp)
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod native {
    use super::*;
    use std::{
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        time::{Duration, Instant},
    };
    use xgpui_media::VideoStream;
    pub struct Demo {
        pub stream: VideoStream,
        stop: Arc<AtomicBool>,
        burst: mpsc::SyncSender<()>,
        error: Arc<Mutex<Option<String>>>,
        worker: Option<std::thread::JoinHandle<()>>,
    }
    impl Demo {
        pub fn new(format: SourceFormat) -> Result<Self> {
            let (sender, stream) = VideoStream::channel();
            sender.publish(format.video_frame(0)?)?;
            let stop = Arc::new(AtomicBool::new(false));
            let done = stop.clone();
            let error = Arc::new(Mutex::new(None));
            let report = error.clone();
            let (burst, requests) = mpsc::sync_channel(1);
            let worker = std::thread::Builder::new().name("media-lab-video".into()).spawn(move || {
                let mut index = 1;
                while !done.load(Ordering::Relaxed) {
                    let start = Instant::now();
                    let count = if requests.try_recv().is_ok() { 120 } else { 1 };
                    for _ in 0..count {
                        if done.load(Ordering::Relaxed) {
                            break;
                        }
                        let result = format.video_frame(index).and_then(|f| sender.publish(f));
                        if let Err(e) = result {
                            *report.lock().unwrap() = Some(e.to_string());
                            return;
                        }
                        index += 1;
                    }
                    std::thread::park_timeout(Duration::from_millis(33).saturating_sub(start.elapsed()));
                }
            })?;
            Ok(Self { stream, stop, burst, error, worker: Some(worker) })
        }
    }
    impl Drop for Demo {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(worker) = self.worker.take() {
                worker.thread().unpark();
                if worker.join().is_err() {
                    eprintln!("video source worker panicked during shutdown");
                }
            }
        }
    }
    impl Demo {
        pub fn stop(&self) {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(worker) = &self.worker {
                worker.thread().unpark();
            }
        }
        pub fn burst(&self) {
            let _ = self.burst.try_send(());
        }
        pub fn error(&self) -> Option<String> {
            self.error.lock().ok().and_then(|error| error.clone())
        }
    }
    #[cfg(test)]
    mod lifecycle_tests {
        use super::*;

        #[::core::prelude::v1::test]
        fn source_shutdown_retains_last_frame_and_restart_is_independent() -> Result<()> {
            let demo = Demo::new(SourceFormat::Bgra)?;
            let stream = demo.stream.clone();
            let shown = stream.snapshot().expect("synchronous initial frame");
            stream.set_playing(false);
            drop(demo); // Joins the worker; no delayed producer survives a restart.
            anyhow::ensure!(stream.stats().closed, "source shutdown must close the producer");
            anyhow::ensure!(
                stream.snapshot().unwrap().timestamp() == shown.timestamp(),
                "closed paused stream retains its displayed frame"
            );
            let restarted = Demo::new(SourceFormat::Bgra)?;
            anyhow::ensure!(!restarted.stream.stats().closed, "new source must be live");
            anyhow::ensure!(restarted.stream.snapshot().is_some(), "restart has an initial frame");
            anyhow::ensure!(stream.stats().closed, "restart cannot revive the old stream");
            Ok(())
        }
    }
}
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use native::Demo;

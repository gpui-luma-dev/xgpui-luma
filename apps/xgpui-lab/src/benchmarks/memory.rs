//! File-free report transport for isolated CPU SVG workers.
use super::*;
use std::io::Read;
const REPORT_LIMIT: u64 = 16 * 1024 * 1024;
fn drain(reader: impl Read) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(REPORT_LIMIT + 1).read_to_end(&mut bytes)?;
    Ok(bytes)
}
pub(crate) fn run(args: &[String], cancel: &AtomicBool, progress: &progress::Progress) -> Result<Value> {
    check_cancel(cancel)?;
    let mut command = Command::new(std::env::current_exe()?);
    command.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn().context("starting isolated SVG worker")?;
    let stdout = child.stdout.take().context("missing worker stdout")?;
    let stderr = child.stderr.take().context("missing worker stderr")?;
    // Drain both streams concurrently so either pipe can fill safely.
    let out = std::thread::spawn(move || drain(stdout));
    let progress = progress.clone();
    let err = std::thread::spawn(move || read_stderr(stderr, &progress));
    let status = wait_child(&mut child, args, cancel);
    if status.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let stdout = out.join().map_err(|_| anyhow::anyhow!("SVG stdout reader failed"))??;
    let stderr = err.join().map_err(|_| anyhow::anyhow!("SVG stderr reader failed"))??;
    let status = status?;
    ensure!(status.success(), "SVG worker exited {status}: {}", String::from_utf8_lossy(&stderr));
    ensure!(stdout.len() as u64 <= REPORT_LIMIT, "SVG worker report exceeds limit");
    let report: Value = serde_json::from_slice(&stdout).context("invalid SVG worker report")?;
    validate_report(&report)?;
    Ok(report)
}
fn read_stderr(reader: impl Read, progress: &progress::Progress) -> std::io::Result<Vec<u8>> {
    use std::io::BufRead;
    let mut captured = Vec::new();
    for line in std::io::BufReader::new(reader).lines() {
        let line = line?;
        if let Some(stage) = line.strip_prefix("XGPUI_SVG_PROGRESS ") {
            progress.stage(stage);
        } else if captured.len() < (REPORT_LIMIT as usize) {
            let remaining = (REPORT_LIMIT as usize) - captured.len();
            captured.extend_from_slice(&line.as_bytes()[..line.len().min(remaining)]);
            captured.push(b'\n');
        }
    }
    Ok::<_, std::io::Error>(captured)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn worker_progress_is_consumed_live_and_errors_remain_available() {
        let progress = progress::Progress::default();
        let captured = read_stderr(
            std::io::Cursor::new(b"XGPUI_SVG_PROGRESS Rasterizing sample 2 of 5\nfont warning\n"),
            &progress,
        )
        .unwrap();
        assert!(progress.snapshot().0.contains("sample 2 of 5"));
        assert_eq!(captured, b"font warning\n");
    }
    #[::core::prelude::v1::test]
    fn cancellation_prevents_starting_a_pipe_worker() {
        assert!(run(&[], &AtomicBool::new(true), &progress::Progress::default()).is_err());
    }
}

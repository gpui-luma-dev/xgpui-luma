//! File-free report transport for isolated CPU SVG workers.
use super::*;
use std::io::Read;
const REPORT_LIMIT: u64 = 16 * 1024 * 1024;
fn drain(reader: impl Read) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(REPORT_LIMIT + 1).read_to_end(&mut bytes)?;
    Ok(bytes)
}
pub(crate) fn run(args: &[String], cancel: &AtomicBool) -> Result<Value> {
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
    let err = std::thread::spawn(move || drain(stderr));
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
#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn cancellation_prevents_starting_a_pipe_worker() {
        assert!(run(&[], &AtomicBool::new(true)).is_err());
    }
}

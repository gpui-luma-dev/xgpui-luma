//! Cargo and crates.io integration. Uploads are only reachable in publish mode.
use crate::{CrateReport, Driver, PACKAGES, Report, payload, sha};
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub struct Registry {
    pub root: PathBuf,
    pub out: PathBuf,
    pub token: String,
    pub source_commit: String,
}
impl Registry {
    fn unchanged(&self) -> Result<()> {
        ensure!(
            git(&self.root, &["rev-parse", "HEAD"])? == self.source_commit
                && git(&self.root, &["status", "--porcelain"])?.is_empty(),
            "Release source changed; stop and rerun from the original clean commit"
        );
        Ok(())
    }
    fn verified_payload(&self, row: &CrateReport) -> Result<()> {
        let metadata = metadata(&self.root)?;
        let target = Path::new(metadata["target_directory"].as_str().context("Missing target directory")?);
        let archive = target.join("package").join(format!("{}-{}.crate", row.name, row.version));
        ensure!(
            payload(&fs::read(archive)?, &row.name, &row.version)?
                == payload(&fs::read(&row.archive)?, &row.name, &row.version)?,
            "{}: verified archive differs from prepared candidate",
            row.name
        );
        Ok(())
    }
    fn redact(&self, text: &str) -> String {
        if self.token.is_empty() {
            text.into()
        } else {
            text.replace(&self.token, "[REDACTED]")
        }
    }
    /// Capture Cargo output without persisting credentials; preserve diagnostics and exit status.
    pub fn cargo(&self, label: &str, args: &[String], upload: bool) -> Result<(i32, String, String)> {
        let log = self.out.join(format!("{label}.log"));
        let raw = tempfile::tempfile()?;
        let mut command = Command::new("cargo");
        command
            .current_dir(&self.root)
            .args(args)
            .env_remove("CARGO_REGISTRY_TOKEN")
            .env_remove("CARGO_REGISTRIES_CRATES_IO_TOKEN")
            .env("CARGO_TERM_COLOR", "never")
            .stdout(Stdio::from(raw.try_clone()?))
            .stderr(Stdio::from(raw.try_clone()?));
        if upload {
            command.env("CARGO_REGISTRY_TOKEN", &self.token);
        }
        println!("cargo {}", args.join(" "));
        let mut child = command.spawn()?;
        let start = Instant::now();
        let code = loop {
            if let Some(status) = child.try_wait()? {
                break status.code().unwrap_or(1);
            }
            if start.elapsed() > Duration::from_secs(1200) {
                child.kill()?;
                child.wait()?;
                break 124;
            }
            thread::sleep(Duration::from_millis(100));
        };
        use std::io::{Read, Seek, SeekFrom};
        let mut raw = raw;
        raw.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        raw.read_to_end(&mut bytes)?;
        let mut text = self.redact(&String::from_utf8_lossy(&bytes));
        if code == 124 {
            text.push_str("\nTimed out; inspect registry state before retrying.\n");
        }
        print!("{text}");
        fs::write(&log, &text)?;
        Ok((code, text, log.to_string_lossy().into_owned()))
    }
    fn fetch(&self, url: &str, missing_ok: bool) -> Result<Option<Vec<u8>>> {
        let file = tempfile::NamedTempFile::new()?;
        let output = Command::new("curl")
            .args([
                "--silent",
                "--show-error",
                "--location",
                "--retry",
                "3",
                "--connect-timeout",
                "15",
                "--max-time",
                "60",
                "--user-agent",
                "xgpui-luma-release",
                "--output",
            ])
            .arg(file.path())
            .args(["--write-out", "%{http_code}", url])
            .output()
            .context("curl is required for crates.io lookups")?;
        ensure!(output.status.success(), "Registry request failed: {}", String::from_utf8_lossy(&output.stderr));
        let status = String::from_utf8_lossy(&output.stdout);
        if status == "404" && missing_ok {
            return Ok(None);
        }
        ensure!(status == "200", "Registry HTTP {status}: {url}");
        Ok(Some(fs::read(file.path())?))
    }
    pub fn prepare(&self, report: &mut Report) -> Result<()> {
        let mut args: Vec<String> =
            ["package", "--locked", "--no-verify", "--registry", "crates-io"].map(str::to_owned).into();
        if report.dirty {
            args.push("--allow-dirty".into());
        }
        for name in PACKAGES {
            args.extend(["-p".into(), name.into()]);
        }
        let (code, _, log) = self.cargo("package", &args, false)?;
        ensure!(code == 0, "Archive generation failed; see {log}");
        fs::create_dir_all(self.out.join("prepared"))?;
        // metadata records Cargo's actual target directory, including overrides.
        let metadata = metadata(&self.root)?;
        let target = Path::new(metadata["target_directory"].as_str().context("Missing target directory")?);
        for row in &mut report.packages {
            row.logs.push(log.clone());
            let filename = format!("{}-{}.crate", row.name, row.version);
            let bytes = fs::read(target.join("package").join(&filename))?;
            payload(&bytes, &row.name, &row.version)?;
            let destination = self.out.join("prepared").join(filename);
            fs::write(&destination, &bytes)?;
            row.archive = destination.to_string_lossy().into_owned();
            row.archive_sha256 = sha(&bytes);
            row.status = "prepared".into();
        }
        Ok(())
    }
}
impl Driver for Registry {
    fn existing(&mut self, row: &mut CrateReport) -> Result<bool> {
        let Some(bytes) = self.fetch(&format!("https://crates.io/api/v1/crates/{}/{}", row.name, row.version), true)?
        else {
            return Ok(false);
        };
        let value: Value = serde_json::from_slice(&bytes)?;
        let entry = &value["version"];
        ensure!(
            entry["crate"] == row.name && entry["num"] == row.version && entry["yanked"] == false,
            "{}: registry identity mismatch or yanked version",
            row.name
        );
        let archive = self
            .fetch(&format!("https://static.crates.io/crates/{0}/{0}-{1}.crate", row.name, row.version), false)?
            .context("Registry archive missing")?;
        let checksum = sha(&archive);
        ensure!(entry["checksum"] == checksum, "{}: registry checksum mismatch", row.name);
        ensure!(
            payload(&archive, &row.name, &row.version)? == payload(&fs::read(&row.archive)?, &row.name, &row.version)?,
            "{} {}: version conflict; published source differs, bump the version",
            row.name,
            row.version
        );
        row.registry_checksum = Some(checksum);
        Ok(true)
    }
    fn verify(&mut self, row: &mut CrateReport) -> Result<()> {
        let mut args: Vec<String> =
            ["publish", "--dry-run", "--locked", "--registry", "crates-io", "-p"].map(str::to_owned).into();
        args.push(row.name.clone());
        // Dirty candidates can be inspected, but publication always requires a clean commit.
        if !git(&self.root, &["status", "--porcelain"])?.is_empty() {
            args.push("--allow-dirty".into());
        }
        let (code, _, log) = self.cargo(&format!("{}-verify", row.name), &args, false)?;
        row.logs.push(log.clone());
        ensure!(code == 0, "{}: registry-backed verification failed; see {log}", row.name);
        self.verified_payload(row)
    }
    fn upload(&mut self, row: &mut CrateReport) -> Result<()> {
        self.unchanged()?;
        self.verified_payload(row)?;
        // Never blindly repeat an upload: each invocation checks the registry first.
        let args = vec![
            "publish".into(),
            "--locked".into(),
            "--registry".into(),
            "crates-io".into(),
            "--config".into(),
            "registries.crates-io.credential-provider=\"cargo:token\"".into(),
            "-p".into(),
            row.name.clone(),
        ];
        let (code, _, log) = self.cargo(&format!("{}-upload", row.name), &args, true)?;
        row.logs.push(log.clone());
        for poll in 0..13 {
            if self.existing(row)? {
                return Ok(());
            }
            if poll < 12 {
                thread::sleep(Duration::from_secs(5));
            }
        }
        bail!(
            "{}: upload unverified (Cargo exit {code}); inspect {log} and rerun this same commit. No automatic re-upload was attempted",
            row.name
        )
    }
    fn save(&mut self, report: &Report) -> Result<()> {
        let mut safe = report.clone();
        safe.error = safe.error.map(|error| self.redact(&error));
        for row in &mut safe.packages {
            row.error = row.error.as_ref().map(|error| self.redact(error));
        }
        let mut temporary = tempfile::NamedTempFile::new_in(&self.out)?;
        temporary.write_all(&serde_json::to_vec_pretty(&safe)?)?;
        temporary.persist(self.out.join("report.json"))?;
        for row in &safe.packages {
            fs::write(self.out.join(format!("{}.json", row.name)), serde_json::to_vec_pretty(row)?)?;
        }
        Ok(())
    }
}
pub fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").current_dir(root).args(args).output()?;
    ensure!(output.status.success(), "git {} failed", args.join(" "));
    Ok(String::from_utf8(output.stdout)?.trim().into())
}
pub fn metadata(root: &Path) -> Result<Value> {
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
        .output()?;
    ensure!(output.status.success(), "Cargo metadata failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(serde_json::from_slice(&output.stdout)?)
}
pub fn inventory(root: &Path, mode: crate::Mode, commit: String, dirty: bool) -> Result<Report> {
    let metadata = metadata(root)?;
    let packages = metadata["packages"].as_array().context("Missing packages")?;
    let mut rows = Vec::new();
    let mut version = None;
    for (index, name) in PACKAGES.iter().enumerate() {
        let package = packages.iter().find(|p| p["name"] == *name).with_context(|| format!("Missing {name}"))?;
        let current = package["version"].as_str().context("Missing version")?;
        ensure!(version.as_deref().is_none_or(|v| v == current), "All five library versions must match");
        version = Some(current.to_owned());
        ensure!(package["publish"] == serde_json::json!(["crates-io"]), "{name}: publication must target crates-io");
        let mut dependencies = Vec::new();
        for dep in package["dependencies"].as_array().context("Missing dependencies")? {
            if dep["kind"] == "dev" {
                continue;
            }
            let dep_name = dep["name"].as_str().context("Missing dependency name")?;
            if let Some(position) = PACKAGES.iter().position(|p| *p == dep_name) {
                ensure!(position < index, "{name}: invalid publication order");
                dependencies.push(dep_name.into());
            } else {
                ensure!(
                    dep["source"].as_str().is_some_and(|s| s.starts_with("registry+")),
                    "{name}: external dependency {dep_name} must come from a registry"
                );
            }
        }
        rows.push(CrateReport {
            name: (*name).into(),
            version: current.into(),
            dependencies,
            status: "pending".into(),
            error: None,
            archive: String::new(),
            archive_sha256: String::new(),
            registry_checksum: None,
            blocked_by: Vec::new(),
            logs: Vec::new(),
        });
    }
    Ok(Report {
        source_commit: commit,
        dirty,
        version: version.context("Missing version")?,
        mode,
        status: "preparing".into(),
        release_ready: false,
        error: None,
        packages: rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reports_redact_credentials_in_global_and_per_crate_errors() {
        let temp = tempfile::tempdir().unwrap();
        let mut registry = Registry {
            root: temp.path().into(),
            out: temp.path().into(),
            token: "test-secret".into(),
            source_commit: "commit".into(),
        };
        let report: Report = serde_json::from_value(serde_json::json!({
            "source_commit":"commit", "dirty":false, "version":"1.0.1", "mode":"plan",
            "status":"failed", "release_ready":false, "error":"failure with test-secret",
            "packages":[{"name":"sdk","version":"1.0.1","dependencies":[],"status":"failed",
                "error":"test-secret rejected", "archive":"unused","archive_sha256":"unused",
                "registry_checksum":null,"blocked_by":[],"logs":[]}]
        }))
        .unwrap();
        registry.save(&report).unwrap();
        for filename in ["report.json", "sdk.json"] {
            let text = fs::read_to_string(temp.path().join(filename)).unwrap();
            assert!(!text.contains("test-secret"));
            assert!(text.contains("[REDACTED]"));
            serde_json::from_str::<Value>(&text).unwrap();
        }
        // Atomic replacement must work on repeat saves, including Windows.
        registry.save(&report).unwrap();
    }
}

//! Registry release planning and resumable publication for the five Luma libraries.
use anyhow::{Context, Result, bail, ensure};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, io::Read, path::Component};

pub mod registry;

pub const PACKAGES: [&str; 5] = [
    "xgpui-luma",
    "xgpui-luma-color",
    "xgpui-luma-look-radix",
    "xgpui-luma-look-shadcn",
    "xgpui-luma-plot",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    Plan,
    DryRun,
    Publish,
}
impl Mode {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "plan" => Ok(Self::Plan),
            "dry-run" => Ok(Self::DryRun),
            "publish" => Ok(Self::Publish),
            _ => bail!("Unknown mode {value}; use plan, dry-run, or publish"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrateReport {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub status: String,
    pub error: Option<String>,
    pub archive: String,
    pub archive_sha256: String,
    pub registry_checksum: Option<String>,
    pub blocked_by: Vec<String>,
    pub logs: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub source_commit: String,
    pub dirty: bool,
    pub version: String,
    pub mode: Mode,
    pub status: String,
    pub release_ready: bool,
    pub error: Option<String>,
    pub packages: Vec<CrateReport>,
}

/// Publishing requires an explicitly selected clean commit and upload credentials.
pub fn publication_guard(mode: Mode, commit: &str, dirty: bool, token: &str) -> Result<()> {
    ensure!(!commit.is_empty(), "Missing source commit");
    if mode == Mode::Publish {
        ensure!(!dirty, "Publishing requires a clean committed source tree");
        ensure!(!token.is_empty(), "Publication requires CARGO_REGISTRY_TOKEN");
    }
    Ok(())
}

pub fn sha(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

/// Compare package source and normalized manifests, excluding Cargo-generated state.
pub fn payload(data: &[u8], name: &str, version: &str) -> Result<BTreeMap<String, Vec<u8>>> {
    let prefix = format!("{name}-{version}");
    let mut files = BTreeMap::new();
    for entry in tar::Archive::new(GzDecoder::new(data)).entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        ensure!(path.components().all(|c| matches!(c, Component::Normal(_))), "Unsafe archive path");
        ensure!(
            entry.header().entry_type().is_file() || entry.header().entry_type().is_dir(),
            "Unexpected archive entry type"
        );
        let relative = path.strip_prefix(&prefix).context("Unexpected archive root")?;
        if entry.header().entry_type().is_file() {
            let key = relative.to_str().context("Non-UTF8 archive path")?.replace('\\', "/");
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            ensure!(files.insert(key, bytes).is_none(), "Duplicate archive entry");
        }
    }
    let mut manifest: toml::Value =
        toml::from_str(std::str::from_utf8(files.get("Cargo.toml").context("Missing normalized manifest")?)?)?;
    let package = manifest.get("package").and_then(toml::Value::as_table).context("Missing package identity")?;
    ensure!(
        package.get("name").and_then(toml::Value::as_str) == Some(name)
            && package.get("version").and_then(toml::Value::as_str) == Some(version),
        "Archive identity mismatch"
    );
    if let Some(package) = manifest.get_mut("package").and_then(toml::Value::as_table_mut) {
        package.remove("publish");
    }
    for generated in ["Cargo.toml", "Cargo.toml.orig", "Cargo.lock", ".cargo_vcs_info.json"] {
        files.remove(generated);
    }
    files.insert("Cargo.toml".into(), serde_json::to_vec(&manifest)?);
    Ok(files)
}

/// Registry operations are injected so failure and resume behavior can be tested offline.
pub trait Driver {
    fn existing(&mut self, row: &mut CrateReport) -> Result<bool>;
    fn verify(&mut self, row: &mut CrateReport) -> Result<()>;
    fn upload(&mut self, row: &mut CrateReport) -> Result<()>;
    fn save(&mut self, report: &Report) -> Result<()>;
}

fn transition(report: &mut Report, driver: &mut impl Driver) -> Result<()> {
    let mut conflict = false;
    // Scan all candidates before the first upload, including already-published dependents.
    for row in &mut report.packages {
        row.status = "checking-registry".into();
        match driver.existing(row) {
            Ok(true) => row.status = "already-published".into(),
            Ok(false) => row.status = "publish-needed".into(),
            Err(error) => {
                row.status = "version-conflict-or-registry-error".into();
                row.error = Some(format!("{error:#}"));
                conflict = true;
                report.error = Some(format!("{error:#}"));
            }
        }
    }
    driver.save(report)?;
    ensure!(!conflict, "Registry checks failed; resolve conflicts/errors before any upload");
    if report.mode == Mode::Plan {
        report.status = "planned".into();
        return Ok(());
    }
    for i in 0..report.packages.len() {
        if report.packages[i].status == "already-published" {
            continue;
        }
        let pending: Vec<String> = report.packages[i]
            .dependencies
            .iter()
            .filter(|dep| {
                !report.packages.iter().any(|row| {
                    row.name == **dep && matches!(row.status.as_str(), "already-published" | "published-verified")
                })
            })
            .cloned()
            .collect();
        if !pending.is_empty() {
            report.packages[i].blocked_by = pending;
            report.packages[i].status = "blocked-unpublished-dependencies".into();
            driver.save(report)?;
            ensure!(report.mode == Mode::DryRun, "Internal dependencies are not published");
            continue;
        }
        report.packages[i].status = "verifying".into();
        driver.save(report)?;
        driver.verify(&mut report.packages[i])?;
        report.packages[i].status = "verified-not-uploaded".into();
        driver.save(report)?;
        if report.mode == Mode::Publish {
            report.packages[i].status = "uploading-unverified".into();
            driver.save(report)?;
            // Check visibility again immediately before upload (including interrupted runs).
            if !driver.existing(&mut report.packages[i])? {
                driver.upload(&mut report.packages[i])?;
            }
            report.packages[i].status = "published-verified".into();
            driver.save(report)?;
        }
    }
    report.release_ready = report.mode == Mode::Publish
        && report
            .packages
            .iter()
            .all(|r| matches!(r.status.as_str(), "already-published" | "published-verified"));
    report.status = if report.release_ready {
        "published-verified"
    } else {
        "dry-run-complete"
    }
    .into();
    Ok(())
}

/// Always persist a final report, including partial progress and failure details.
pub fn run(report: &mut Report, driver: &mut impl Driver) -> Result<()> {
    let result = transition(report, driver);
    if let Err(error) = &result {
        report.status = "failed".into();
        let detail = format!("{error:#}");
        report.error = Some(match &report.error {
            Some(prior) => format!("{detail}: {prior}"),
            None => detail,
        });
        for row in &mut report.packages {
            if row.status == "verifying" {
                row.status = "verification-failed".into();
                row.error = report.error.clone();
            } else if row.status == "uploading-unverified" {
                row.status = "upload-unverified; rerun same commit".into();
                row.error = report.error.clone();
            }
        }
    }
    driver.save(report)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[derive(Default)]
    struct Fake {
        published: BTreeSet<String>,
        conflicts: BTreeSet<String>,
        uploads: Vec<String>,
        verifies: Vec<String>,
        saves: Vec<Report>,
        fail_upload: Option<String>,
        fail_verify: Option<String>,
        ambiguous_upload: bool,
    }
    impl Driver for Fake {
        fn existing(&mut self, row: &mut CrateReport) -> Result<bool> {
            ensure!(!self.conflicts.contains(&row.name), "{}: published source differs", row.name);
            Ok(self.published.contains(&row.name))
        }
        fn verify(&mut self, row: &mut CrateReport) -> Result<()> {
            self.verifies.push(row.name.clone());
            ensure!(self.fail_verify.as_ref() != Some(&row.name), "verification failed");
            Ok(())
        }
        fn upload(&mut self, row: &mut CrateReport) -> Result<()> {
            self.uploads.push(row.name.clone());
            if self.fail_upload.as_ref() == Some(&row.name) {
                if self.ambiguous_upload {
                    self.published.insert(row.name.clone());
                }
                bail!("upload interrupted");
            }
            self.published.insert(row.name.clone());
            Ok(())
        }
        fn save(&mut self, report: &Report) -> Result<()> {
            self.saves.push(report.clone());
            Ok(())
        }
    }
    fn report(mode: Mode) -> Report {
        Report {
            source_commit: "commit".into(),
            dirty: false,
            version: "1.0.1".into(),
            mode,
            status: "pending".into(),
            release_ready: false,
            error: None,
            packages: ["sdk", "color"]
                .iter()
                .enumerate()
                .map(|(i, name)| CrateReport {
                    name: (*name).into(),
                    version: "1.0.1".into(),
                    dependencies: if i == 1 { vec!["sdk".into()] } else { vec![] },
                    status: "prepared".into(),
                    archive: "unused".into(),
                    archive_sha256: "unused".into(),
                    registry_checksum: None,
                    error: None,
                    blocked_by: vec![],
                    logs: vec![],
                })
                .collect(),
        }
    }
    #[test]
    fn plan_never_verifies_or_uploads() {
        let mut fake = Fake::default();
        let mut r = report(Mode::Plan);
        run(&mut r, &mut fake).unwrap();
        assert!(fake.uploads.is_empty() && fake.verifies.is_empty());
        assert_eq!(r.status, "planned");
        assert!(!r.release_ready);
    }
    #[test]
    fn dry_run_reports_unpublished_dependencies_without_upload() {
        let mut fake = Fake::default();
        let mut r = report(Mode::DryRun);
        run(&mut r, &mut fake).unwrap();
        assert_eq!(fake.verifies, ["sdk"]);
        assert!(fake.uploads.is_empty());
        assert_eq!(r.packages[1].status, "blocked-unpublished-dependencies");
        assert_eq!(r.packages[1].blocked_by, ["sdk"]);
    }
    #[test]
    fn dry_run_verifies_dependents_when_parent_is_published() {
        let mut fake = Fake::default();
        fake.published.insert("sdk".into());
        run(&mut report(Mode::DryRun), &mut fake).unwrap();
        assert_eq!(fake.verifies, ["color"]);
        assert!(fake.uploads.is_empty());
    }
    #[test]
    fn collision_in_later_crate_blocks_all_uploads() {
        let mut fake = Fake::default();
        fake.conflicts.insert("color".into());
        let mut r = report(Mode::Publish);
        assert!(run(&mut r, &mut fake).is_err());
        assert!(fake.uploads.is_empty());
        assert_eq!(r.status, "failed");
        assert!(r.error.unwrap().contains("published source differs"));
    }
    #[test]
    fn publish_orders_dependencies_and_marks_readiness() {
        let mut fake = Fake::default();
        let mut r = report(Mode::Publish);
        run(&mut r, &mut fake).unwrap();
        assert_eq!(fake.uploads, ["sdk", "color"]);
        assert!(r.release_ready);
    }
    #[test]
    fn resumes_partial_release_by_registry_state() {
        let mut fake = Fake { fail_upload: Some("color".into()), ..Fake::default() };
        let mut first = report(Mode::Publish);
        assert!(run(&mut first, &mut fake).is_err());
        assert_eq!(first.packages[0].status, "published-verified");
        assert!(first.packages[1].status.starts_with("upload-unverified"));
        fake.fail_upload = None;
        fake.uploads.clear();
        let mut second = report(Mode::Publish);
        run(&mut second, &mut fake).unwrap();
        assert_eq!(fake.uploads, ["color"]);
        assert_eq!(second.packages[0].status, "already-published");
    }
    #[test]
    fn ambiguous_upload_is_not_repeated_on_resume() {
        let mut fake = Fake { fail_upload: Some("sdk".into()), ambiguous_upload: true, ..Fake::default() };
        assert!(run(&mut report(Mode::Publish), &mut fake).is_err());
        fake.fail_upload = None;
        fake.uploads.clear();
        run(&mut report(Mode::Publish), &mut fake).unwrap();
        assert_eq!(fake.uploads, ["color"]);
    }
    #[test]
    fn verification_failure_never_uploads_and_is_saved() {
        let mut fake = Fake { fail_verify: Some("sdk".into()), ..Fake::default() };
        let mut r = report(Mode::Publish);
        assert!(run(&mut r, &mut fake).is_err());
        assert!(fake.uploads.is_empty());
        assert_eq!(fake.saves.last().unwrap().packages[0].status, "verification-failed");
    }
    #[test]
    fn publish_requires_clean_commit_and_token_but_plan_does_not() {
        assert!(publication_guard(Mode::Publish, "commit", false, "secret").is_ok());
        assert!(publication_guard(Mode::Publish, "commit", true, "secret").is_err());
        assert!(publication_guard(Mode::Publish, "commit", false, "").is_err());
        assert!(publication_guard(Mode::Plan, "commit", true, "").is_ok());
        assert!(publication_guard(Mode::DryRun, "commit", true, "").is_ok());
        assert!(Mode::parse("invalid").is_err());
    }
    fn archive(name: &str, version: &str, code: &str, lock: &str) -> Vec<u8> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut tar = tar::Builder::new(encoder);
        let manifest = format!("[package]\nname = {name:?}\nversion = {version:?}\n");
        for (file, text) in [("Cargo.toml", manifest.as_str()), ("src/lib.rs", code), ("Cargo.lock", lock)] {
            let mut header = tar::Header::new_gnu();
            header.set_size(text.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, format!("{name}-{version}/{file}"), text.as_bytes()).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap()
    }
    #[test]
    fn archive_comparison_ignores_generated_lock_but_preserves_source() {
        let one = archive("sdk", "1.0.1", "original", "lock1");
        let two = archive("sdk", "1.0.1", "original", "lock2");
        assert_eq!(payload(&one, "sdk", "1.0.1").unwrap(), payload(&two, "sdk", "1.0.1").unwrap());
        let changed = archive("sdk", "1.0.1", "modified", "lock1");
        assert_ne!(payload(&one, "sdk", "1.0.1").unwrap(), payload(&changed, "sdk", "1.0.1").unwrap());
        assert!(payload(&one, "sdk", "9.0.0").is_err());
    }
}

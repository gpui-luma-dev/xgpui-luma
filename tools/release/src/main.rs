use anyhow::{Context, Result, bail, ensure};
use luma_release::{
    Driver, Mode, Report, publication_guard,
    registry::{Registry, git, inventory},
    run,
};
use std::{
    env, fs,
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() {
    if let Err(error) = execute() {
        let token = token();
        let text = format!("{error:#}");
        eprintln!(
            "{}",
            if token.is_empty() {
                text
            } else {
                text.replace(&token, "[REDACTED]")
            }
        );
        std::process::exit(1);
    }
}
fn token() -> String {
    env::var("CARGO_REGISTRY_TOKEN")
        .or_else(|_| env::var("CARGO_REGISTRIES_CRATES_IO_TOKEN"))
        .unwrap_or_default()
}
fn execute() -> Result<()> {
    let mut mode = Mode::Publish;
    let mut out = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mode" => mode = Mode::parse(&args.next().context("--mode requires plan, dry-run, or publish")?)?,
            "--out" => out = Some(PathBuf::from(args.next().context("--out requires a directory")?)),
            "--help" | "-h" => {
                println!(
                    "Usage: just publish-crates [--mode plan|dry-run|publish] [--out DIRECTORY]\n\nDefault: publish. Uploads require a clean commit and CARGO_REGISTRY_TOKEN\n(or CARGO_REGISTRIES_CRATES_IO_TOKEN). Plan and dry-run never upload.\nResume a partial release by rerunning the same commit; matching published crates are skipped.\nReports and archives: target/release-report/<commit>/<attempt>/"
                );
                return Ok(());
            }
            _ => bail!("Unknown argument {arg}; see --help"),
        }
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
    let commit = git(&root, &["rev-parse", "HEAD"])?;
    let dirty = !git(&root, &["status", "--porcelain"])?.is_empty();
    let token = token();
    publication_guard(mode, &commit, dirty, &token)?;
    let mut report = inventory(&root, mode, commit.clone(), dirty)?;
    let attempt = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let out = out.unwrap_or_else(|| root.join("target/release-report").join(&commit).join(attempt.to_string()));
    ensure!(!out.exists(), "Output directory already exists; choose a new --out to retain earlier diagnostics");
    fs::create_dir_all(&out)?;
    let out = out.canonicalize()?;
    let mut registry = Registry { root, out: out.clone(), token, source_commit: commit };
    registry.save(&report)?;
    let result = (|| {
        registry.prepare(&mut report)?;
        run(&mut report, &mut registry)
    })();
    if let Err(error) = &result {
        report.status = "failed".into();
        if report.error.is_none() {
            report.error = Some(format!("{error:#}"));
        }
    }
    if !registry.token.is_empty() {
        report.error = report.error.map(|error| error.replace(&registry.token, "[REDACTED]"));
        for row in &mut report.packages {
            row.error = row.error.as_ref().map(|error| error.replace(&registry.token, "[REDACTED]"));
        }
    }
    registry.save(&report)?;
    summary(&report, &out)?;
    if let Ok(path) = env::var("GITHUB_OUTPUT") {
        let mut output = fs::OpenOptions::new().append(true).open(path)?;
        writeln!(
            output,
            "version={}\ntag=v{}\nreport_dir={}\nrelease_ready={}",
            report.version,
            report.version,
            out.display(),
            report.release_ready
        )?;
    }
    result
}
fn summary(report: &Report, out: &std::path::Path) -> Result<()> {
    println!("\n==== crates.io {} ({:?}) ====", report.version, report.mode);
    let mut markdown = format!(
        "## crates.io {}\n\nMode: `{:?}`. Commit: `{}`. Status: `{}`.\n\n| Crate | Status |\n|---|---|\n",
        report.version, report.mode, report.source_commit, report.status
    );
    for row in &report.packages {
        println!("{:<30} {}", row.name, row.status);
        markdown.push_str(&format!("| {} | {} |\n", row.name, row.status));
    }
    println!("Reports, logs and prepared archives: {}", out.display());
    if let Some(error) = &report.error {
        println!("Failure: {error}");
    }
    if let Ok(path) = env::var("GITHUB_STEP_SUMMARY") {
        fs::OpenOptions::new().append(true).open(path)?.write_all(markdown.as_bytes())?;
    }
    Ok(())
}

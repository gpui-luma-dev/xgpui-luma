#![cfg(feature = "build-support")]

use std::{
    fs,
    process::{Command, Output},
};
use tempfile::TempDir;

fn run(args: &[&str], project: &TempDir) -> Output {
    Command::new(env!("CARGO_BIN_EXE_luma-l10n-check"))
        .args(args)
        .current_dir(project.path())
        .output()
        .unwrap()
}

#[test]
fn checker_validates_locales_usage_and_exit_status() {
    let project = TempDir::new().unwrap();
    let root = project.path();
    fs::write(root.join("en.ftl"), "base = Hello { $name }\nwrapper = { base }\nunused = Text").unwrap();
    fs::write(root.join("de.ftl"), "base = Hallo { $name }\nwrapper = { base }\nunused = Text").unwrap();
    fs::write(root.join("used.txt"), "# Used IDs\nwrapper\nunused\n").unwrap();
    let args = ["--reference", "en.ftl", "--locale", "de=de.ftl", "--used", "used.txt"];
    let result = run(&args, &project);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));

    fs::write(root.join("de.ftl"), "base = Hallo { $wrong }\nextra = Extra").unwrap();
    fs::write(root.join("used.txt"), "wrapper").unwrap();
    let result = run(&args, &project);
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(!result.status.success());
    for expected in [
        "argument mismatch for base",
        "missing message: wrapper",
        "unknown message: extra",
        "unused message: unused",
    ] {
        assert!(stderr.contains(expected), "{stderr}");
    }

    fs::write(root.join("used.txt"), "typo").unwrap();
    let result = run(&["--reference", "en.ftl", "--used", "used.txt"], &project);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("used message missing from reference catalog: typo"));
}

#[test]
fn checker_rejects_malformed_files_invalid_locale_tags_and_bad_arguments() {
    let project = TempDir::new().unwrap();
    fs::write(project.path().join("en.ftl"), "title = Hello").unwrap();
    fs::write(project.path().join("bad.ftl"), "title = {").unwrap();
    for args in [
        vec![],
        vec!["--reference"],
        vec!["--reference", "missing.ftl"],
        vec!["--unknown"],
        vec!["--reference", "en.ftl", "--locale", "bad!=bad.ftl"],
        vec!["--reference", "en.ftl", "--locale", "de=bad.ftl"],
        vec!["--reference", "en.ftl", "--locale", "de"],
        vec!["--reference", "en.ftl", "--locale", "de=en.ftl", "--locale", "DE=en.ftl"],
        vec!["--reference", "en.ftl", "--reference", "en.ftl"],
    ] {
        assert!(!run(&args, &project).status.success(), "{args:?}");
    }
    assert!(run(&["--help"], &project).status.success());
}

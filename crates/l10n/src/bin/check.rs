//! Headless resource checks; see `--help` for usage.
use std::{collections::BTreeSet, env, fs, path::PathBuf};
use xgpui_luma_l10n::{Locale, build_support::Catalog};

const USAGE: &str = "Usage: luma-l10n-check --reference EN.ftl [--locale TAG=FILE.ftl ...] [--used IDs.txt]\n\nChecks Fluent syntax, references, argument parity, and translation coverage.\n--used checks code-used IDs against English and reports unused messages.\nIDs.txt contains one ID or id.attribute per line; blank lines and # comments are ignored.\nAll reported differences (including unused messages) produce a nonzero exit status.";

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut reference = None;
    let mut used = None;
    let mut translations = Vec::new();
    let mut locales = BTreeSet::new();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(());
            }
            "--reference" | "--used" => {
                let value = args.next().ok_or_else(|| format!("{arg} needs a path"))?;
                let slot = if arg == "--reference" {
                    &mut reference
                } else {
                    &mut used
                };
                if slot.replace(PathBuf::from(value)).is_some() {
                    return Err(format!("duplicate {arg}").into());
                }
            }
            "--locale" => {
                let value = args.next().ok_or("--locale needs TAG=FILE.ftl")?;
                let (tag, path) = value.split_once('=').ok_or("--locale needs TAG=FILE.ftl")?;
                let locale: Locale = tag.parse()?;
                if !locales.insert(locale.to_string()) {
                    return Err(format!("duplicate locale {locale}").into());
                }
                translations.push((locale, PathBuf::from(path)));
            }
            _ => return Err(format!("unknown argument {arg}\n{USAGE}").into()),
        }
    }
    let reference = reference.ok_or(USAGE)?;
    let source = fs::read_to_string(&reference)?;
    let catalog = Catalog::from_sources([(reference.to_str().ok_or("non-UTF8 reference path")?, source.as_str())])?;
    if catalog.messages().is_empty() {
        return Err("reference catalog has no message values or attributes".into());
    }
    let mut failed = false;
    for (locale, path) in translations {
        let source = fs::read_to_string(&path)?;
        let translation = Catalog::from_sources([(path.to_str().ok_or("non-UTF8 locale path")?, source.as_str())])?;
        for diagnostic in catalog.compare(&translation) {
            eprintln!("{locale}: {diagnostic}");
            failed = true;
        }
    }
    if let Some(path) = used {
        let source = fs::read_to_string(path)?;
        let ids = source.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#'));
        for id in catalog.unused(ids)? {
            eprintln!("unused message: {id}");
            failed = true;
        }
    }
    if failed {
        return Err("localization resource checks failed".into());
    }
    println!("localization resources checked successfully");
    Ok(())
}

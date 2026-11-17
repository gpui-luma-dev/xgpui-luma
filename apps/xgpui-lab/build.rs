use std::{env, error::Error, fs, path::PathBuf};
use xgpui_luma_l10n::build_support::Catalog;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=locales");
    let english = fs::read_to_string("locales/en/lab.ftl")?;
    let catalog = Catalog::from_sources([("locales/en/lab.ftl", english.as_str())])?;
    for locale in ["de", "ja", "ar"] {
        let path = format!("locales/{locale}/lab.ftl");
        let source = fs::read_to_string(&path)?;
        let translation = Catalog::from_sources([(path.as_str(), source.as_str())])?;
        let differences = catalog.compare(&translation);
        if !differences.is_empty() {
            return Err(format!("{locale}: {differences:?}").into());
        }
    }
    let used = fs::read_to_string("locales/used-ids.txt")?;
    let unused = catalog.unused(used.lines().map(str::trim).filter(|id| !id.is_empty() && !id.starts_with('#')))?;
    if !unused.is_empty() {
        return Err(format!("unused lab messages: {unused:?}").into());
    }
    let output = PathBuf::from(env::var("OUT_DIR")?).join("lab_messages.rs");
    fs::write(output, catalog.generate_rust()?)?;
    Ok(())
}

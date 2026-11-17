#[path = "localization/pseudo.rs"]
mod pseudo;
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
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let entries: Vec<serde_json::Value> = serde_json::from_str(&fs::read_to_string("locales/copy.json")?)?;
    let mut bridge = String::from("use xgpui_luma_l10n::Message;\nuse super::super::messages;\n");
    bridge.push_str("pub fn literal(source: &str) -> Option<Message<'static>> { match source {\n");
    for entry in &entries {
        let source = entry["source"].as_str().ok_or("copy source")?;
        let id = entry["id"].as_str().ok_or("copy id")?;
        if entry["parts"].is_null() {
            bridge.push_str(&format!("{source:?} => Some(messages::msg_{}()),\n", id.replace('-', "_")));
        }
    }
    bridge.push_str("_ => None, } }\n");
    fs::write(out_dir.join("lab_copy.rs"), bridge)?;
    for (tag, rtl) in [("en-XA", false), ("ar-XB", true)] {
        let source = pseudo::resource(&english, rtl)?;
        let pseudo_catalog = Catalog::from_sources([(tag, source.as_str())])?;
        if !catalog.compare(&pseudo_catalog).is_empty() {
            return Err(format!("{tag}: pseudo resource schema changed").into());
        }
        fs::write(out_dir.join(format!("{tag}-lab.ftl")), source)?;
        let sdk = fs::read_to_string("../../crates/l10n/locales/en/sdk.ftl")?;
        fs::write(out_dir.join(format!("{tag}-sdk.ftl")), pseudo::resource(&sdk, rtl)?)?;
    }
    println!("cargo:rerun-if-changed=../../crates/l10n/locales/en/sdk.ftl");
    println!("cargo:rerun-if-changed=localization");
    let output = out_dir.join("lab_messages.rs");
    fs::write(output, catalog.generate_rust()?)?;
    Ok(())
}

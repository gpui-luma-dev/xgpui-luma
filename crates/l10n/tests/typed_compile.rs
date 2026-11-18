#![cfg(feature = "build-support")]

use std::{
    fs,
    process::{Command, Output},
};
use tempfile::TempDir;

// Compile a real consumer using the public build.rs API. This catches generated
// Rust/lifetime errors and proves misspelled IDs and missing/extra args fail.
#[test]
fn generated_consumer_compiles_and_enforces_ids_and_argument_presence() {
    let project = TempDir::new().unwrap();
    let root = project.path();
    fs::create_dir(root.join("src")).unwrap();
    let dependency = env!("CARGO_MANIFEST_DIR");
    fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"
[package]
name = "l10n-typed-consumer"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
xgpui-luma-l10n = {{ path = {dependency:?}, default-features = false }}
[build-dependencies]
xgpui-luma-l10n = {{ path = {dependency:?}, default-features = false, features = ["build-support"] }}
"#
        ),
    )
    .unwrap();
    // Reuse the repository's resolved versions without changing its lockfile.
    fs::copy(std::path::Path::new(dependency).join("../../Cargo.lock"), root.join("Cargo.lock")).unwrap();
    fs::write(root.join("en.ftl"), "welcome = Hello { $name }\ncount = { $n } items\nsearch =\n    .placeholder = Search { $query }\nplain = Plain").unwrap();
    fs::write(
        root.join("build.rs"),
        r#"
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("en.ftl")?;
    let catalog = xgpui_luma_l10n::build_support::Catalog::from_sources([("en.ftl", source.as_str())])?;
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("messages.rs");
    std::fs::write(output, catalog.generate_rust()?)?;
    println!("cargo:rerun-if-changed=en.ftl");
    Ok(())
}
"#,
    )
    .unwrap();
    let prefix = "#![deny(warnings)]\nmod messages { include!(concat!(env!(\"OUT_DIR\"), \"/messages.rs\")); }\n";
    fs::write(
        root.join("src/lib.rs"),
        format!(
            "{prefix}{}",
            r#"
#[test]
fn typed_messages_render() {
    let mut l10n = xgpui_luma_l10n::Localizer::new("de".parse().unwrap());
    l10n.add_resource(&"en".parse().unwrap(), include_str!("../en.ftl")).unwrap();
    l10n.add_resource(&"de".parse().unwrap(), "welcome = Hallo { $name }\nsearch = Suche").unwrap();
    let name = String::from("Ada");
    assert_eq!(l10n.format(messages::msg_welcome(name.as_str())), "Hallo Ada");
    assert_eq!(l10n.format(messages::msg_count(3)), "3 items");
    assert_eq!(l10n.format(messages::msg_search_placeholder("text")), "Search text");
    assert_eq!(l10n.format(messages::msg_plain()), "Plain");
    assert_eq!(messages::MESSAGE_IDS.len(), 4);
}
"#
        ),
    )
    .unwrap();
    let output = cargo(&project, "test");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    for (expression, error) in [
        ("messages::msg_typo()", "E0425"),
        ("messages::msg_welcome()", "E0061"),
        ("messages::msg_plain(1)", "E0061"),
        ("messages::msg_welcome(vec![1, 2])", "E0277"),
    ] {
        fs::write(root.join("src/lib.rs"), format!("{prefix}pub fn bad() {{ let _ = {expression}; }}")).unwrap();
        let output = cargo(&project, "check");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success() && stderr.contains(error), "{expression}: {stderr}");
    }
}

fn cargo(project: &TempDir, action: &str) -> Output {
    Command::new(env!("CARGO"))
        .arg(action)
        .args(["--offline", "--quiet", "--manifest-path"])
        .arg(project.path().join("Cargo.toml"))
        .arg("--target-dir")
        .arg(project.path().join("target"))
        .output()
        .unwrap()
}

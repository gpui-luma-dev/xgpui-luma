#![cfg(feature = "build-support")]

use xgpui_luma_l10n::build_support::{Catalog, Diagnostic};

fn catalog(source: &str) -> Catalog {
    Catalog::from_sources([("test.ftl", source)]).unwrap()
}

#[test]
fn contracts_include_selectors_nested_placeables_functions_and_references() {
    let catalog = catalog(
        "\
-brand = { $case ->
    [long] Brand
   *[other] Short
}
base = Hello { $name }
    .label = { $label }
welcome = { base } { base.label } { -brand(case: \"long\") }
count = { $n ->
    [one] { NUMBER($value) }
   *[other] { { $other } }
}
",
    );
    assert_eq!(catalog.messages()["welcome"].iter().map(String::as_str).collect::<Vec<_>>(), ["label", "name"]);
    assert_eq!(catalog.messages()["count"].iter().map(String::as_str).collect::<Vec<_>>(), ["n", "other", "value"]);
    assert!(!catalog.messages().contains_key("-brand"));
    assert_eq!(catalog.unused(["welcome", "count"]).unwrap(), Vec::<String>::new());
}

#[test]
fn rejects_invalid_resources_duplicate_definitions_and_reference_cycles() {
    for source in [
        "broken = {",
        "title = One\ntitle = Two",
        "title = One\n    .label = A\n    .label = B",
        "title = { missing }",
        "title = { -missing }",
        "title = { title }",
        "a = { b }\nb = { a }",
        "-a = { -b }\n-b = { -a }\ntitle = { -a }",
        "a = Text\nb = { a.missing }",
    ] {
        assert!(Catalog::from_sources([("bad.ftl", source)]).is_err(), "{source}");
    }
    let error = Catalog::from_sources([("broken.ftl", "a = {")]).unwrap_err().to_string();
    assert!(error.contains("broken.ftl"));
}

#[test]
fn app_overrides_replace_entire_entries_like_runtime_bundles() {
    let catalog = Catalog::from_sources([
        ("sdk.ftl", "title = SDK { $old }\n    .label = Old"),
        ("app.ftl", "title = App { $new }"),
    ])
    .unwrap();
    assert_eq!(catalog.messages()["title"].iter().map(String::as_str).collect::<Vec<_>>(), ["new"]);
    assert!(!catalog.messages().contains_key("title.label"));
}

#[test]
fn translation_checks_report_missing_unknown_and_mismatched_arguments() {
    let english = catalog("title = Hi { $name }\n    .label = Label\nonly-en = English");
    let translation = catalog("title = Hallo { $user }\nextra = Extra");
    assert_eq!(
        english.compare(&translation),
        vec![
            Diagnostic::MissingMessage("only-en".into()),
            Diagnostic::Arguments { id: "title".into(), expected: vec!["name".into()], actual: vec!["user".into()] },
            Diagnostic::MissingMessage("title.label".into()),
            Diagnostic::UnknownMessage("extra".into()),
        ]
    );
    assert!(
        english
            .compare(&catalog("title = Bonjour { $name }\n    .label = Étiquette\nonly-en = Français"))
            .is_empty()
    );
}

#[test]
fn unused_report_tracks_references_and_rejects_unknown_used_ids() {
    let catalog = catalog("base = Hello\nwrapper = { base }\nunused = Bye\n    .label = Label");
    assert_eq!(catalog.unused(["wrapper"]).unwrap(), ["unused", "unused.label"]);
    assert!(catalog.unused(["typo"]).unwrap_err().to_string().contains("typo"));
}

#[test]
fn generated_names_are_deterministic_and_reject_collisions() {
    let first = catalog("type = Type\nlab-count = { $n }\n").generate_rust().unwrap();
    let second = catalog("lab-count = { $n }\ntype = Type\n").generate_rust().unwrap();
    assert_eq!(first, second);
    assert!(first.contains("fn msg_type"));
    assert!(catalog("a-b = One\na_b = Two").generate_rust().is_err());
    assert!(catalog("a = One\n    .b = B\na-b = Two").generate_rust().is_err());
    assert!(catalog("a = { $a-b } { $a_b }").generate_rust().is_err());
}

#[test]
fn bounds_reference_depth_even_with_cached_parent_contracts() {
    for reversed in [false, true] {
        let mut source = String::from("n000 = Value\n");
        for index in 1..110 {
            if reversed {
                source.push_str(&format!("n{index:03} = {{ n{:03} }}\n", index - 1));
            } else {
                source.push_str(&format!("z{index:03} = {{ z{:03} }}\n", index + 1));
            }
        }
        source.push_str("z110 = Value\n");
        let error = Catalog::from_sources([("deep.ftl", source.as_str())]).unwrap_err();
        assert!(error.to_string().contains("depth exceeds 100"), "{error}");
    }
}

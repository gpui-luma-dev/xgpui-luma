//! Inspection for the generic inspector (API topic 6).

mod common;

use common::*;
use xgpui_luma_style::{Category, Kind, PartId, ReferenceKind, Value, ValueSource};

const INDICATOR: PartId = PartId::new("indicator");

#[test]
fn inspection_covers_every_part_with_rows_docs_and_box_model() {
    let theme = sdk_theme();
    let inspection = theme.inspect(&key(&theme, "checkbox", &[], &["checked"]), None);
    assert_eq!(inspection.control.name(), "checkbox");
    assert_eq!(inspection.doc.as_deref(), Some("An indicator box with a checkmark, and a label."));
    let parts: Vec<_> = inspection.parts.iter().map(|p| p.part.name()).collect();
    assert_eq!(parts, ["root", "indicator", "glyph", "label"]);

    let root = &inspection.parts[0];
    let glyph_size = root.rows.iter().find(|r| r.property == "glyph_size").expect("declared row");
    assert_eq!(glyph_size.kind, Kind::Length);
    assert_eq!(glyph_size.category, Category::Layout);
    assert_eq!(glyph_size.doc.as_deref(), Some("Checkmark size."));
    assert_eq!(glyph_size.explanation.value, Some(Value::Number(12.0)));
    assert_eq!(root.box_model.border_width, 1.0);

    let indicator = &inspection.parts[1];
    let bg = indicator.rows.iter().find(|r| r.property == "background").expect("row");
    assert_eq!(bg.category, Category::Colors);
    assert_eq!(bg.explanation.winner.as_ref().map(|w| w.states.as_str()), Some(":checked"));
}

#[test]
fn text_properties_inherited_from_root_are_marked() {
    // The SDK popup menu sets font_size on the control (root) only; its trigger inherits it.
    let theme = sdk_theme();
    let inspection = theme.inspect(&key(&theme, "popup_menu", &[], &[]), None);
    let trigger = inspection.parts.iter().find(|p| p.part.name() == "trigger").expect("part");
    let fs = trigger.rows.iter().find(|r| r.property == "font_size").expect("inherited row");
    assert_eq!(fs.explanation.source, ValueSource::InheritedFromRoot);
    assert_eq!(fs.explanation.value, Some(Value::Number(12.5))); // text metrics are never snapped
    assert_eq!(fs.category, Category::Typography);
}

#[test]
fn references_trace_a_value_back_to_the_catalog() {
    let theme = shadcn_theme("light");
    let why = theme.explain(&key(&theme, "checkbox", &[], &[]), INDICATOR, "background");
    // background = shadcn.outline_layer.default → first(css.card, css.background) → css.card
    let top = &why.references[0];
    assert_eq!(top.name, "shadcn.outline_layer.default");
    assert_eq!(top.kind, ReferenceKind::Token);
    assert_eq!(top.defined_at.as_ref().map(|l| l.source.as_ref()), Some("shadcn/tokens.lss"));
    let native = &top.children[0];
    assert_eq!(native.name, "css.card");
    assert_eq!(native.kind, ReferenceKind::Native);
    assert!(native.value.is_some());
}

#[test]
fn references_include_properties_and_constants() {
    let theme = sdk_theme();
    let why = theme.explain(&key(&theme, "checkbox", &[], &[]), PartId::ROOT, "glyph_size");
    let names: Vec<_> = why.references.iter().map(|r| (r.name.as_str(), r.kind)).collect();
    assert_eq!(names, [("indicator_size", ReferenceKind::Property), ("glyph_inset", ReferenceKind::Property)]);
    // indicator_size itself read the control-height token and the scale.
    let size = &why.references[0];
    assert!(size.children.iter().any(|c| c.name == "metrics.control.control_height"));
    assert_eq!(size.value, Some(Value::Number(18.0)));
}

#[test]
fn axes_come_from_the_files() {
    let sheet = shadcn_sheet();
    let checkbox = sheet.control("checkbox").expect("declared");
    let axes = sheet.axes(checkbox);
    let config = sheet.config();
    let variant = config.attribute("variant").expect("attribute");
    let size = config.attribute("size").expect("attribute");
    let variants: Vec<_> = axes
        .attributes
        .iter()
        .find(|(a, _)| *a == variant)
        .expect("variants")
        .1
        .iter()
        .map(|v| v.name())
        .collect();
    assert_eq!(variants, ["secondary", "outline", "ghost", "content_only"]);
    let sizes: Vec<_> = axes
        .attributes
        .iter()
        .find(|(a, _)| *a == size)
        .expect("sizes")
        .1
        .iter()
        .map(|v| v.name())
        .collect();
    assert_eq!(sizes, ["sm", "lg"]);
    let states: Vec<_> = axes.states.iter().map(|s| config.state_name(*s)).collect();
    assert_eq!(states, ["hover", "pressed", "disabled", "checked"]);
    assert_eq!(axes.modes, ["light", "dark"], "declaration order");
}

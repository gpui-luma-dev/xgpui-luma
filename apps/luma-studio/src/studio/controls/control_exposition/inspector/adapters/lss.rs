//! One inspector adapter for every control styled with `.lss` (issue #285).
//!
//! It turns the style crate's `Inspection` for the selected variant, size, state and value into this
//! inspector's rows: colors, layout (box model, occupation, sizes), elevation and typography, each with
//! the rule that set it and the tokens it read. A control needs only its spec (the selector lists)
//! and, for an indicator-only template path, the names of its indicator geometry.

use std::sync::Arc;

use gpui::Hsla;
use lucide_svg_static::Icon as LucideIcon;
use xgpui_luma::infra::focus_adorner::FocusAdorner;
use xgpui_luma::style::lss::{
    Category, ControlId, Inspection, PartId, PartInspection, ReferenceKind, Row, StyleKey, Theme, Value, box_shadows,
    display_value,
};
use xgpui_luma::theme::FocusMetricTokens;
use xgpui_luma_look_shadcn::ShadcnLook;

use super::super::box_model::{InspectBoxModelSnapshot, InspectOccupationSnapshot};
use super::super::common::{choice_indicator_only, interaction_state};
use super::super::metrics::layout_section;
use super::super::occupation::{choice_indicator_occupation, occupation_metric_properties};
use super::super::provenance::property_rows_from_metric_data;
use super::super::schema::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectElevationLayer, InspectElevationSnapshot,
    InspectPropertyRow, InspectorCategory, InspectorCategoryContent, InspectorPart, InspectorSelection, InspectorSize,
    InspectorStateSpec, InspectorValueMode, InspectorVariant, SharedInspectorResolver,
};

/// The indicator-only template path: the box model uses these root properties, and these parts are hidden.
#[derive(Clone, Copy, Debug)]
pub struct IndicatorOnly {
    pub size: &'static str,
    pub radius: &'static str,
    pub hidden_parts: &'static [&'static str],
}

/// The labeled box model from named root properties instead of the root's own box (a switch's track).
#[derive(Clone, Copy, Debug)]
pub struct BoxProps {
    pub height: &'static str,
    pub padding: &'static str,
    pub radius: &'static str,
}

/// Inspector rows for one `.lss` control, from the look's current style theme.
pub struct LssInspectorAdapter {
    control: ControlId,
    /// The state the value selector turns on (`"checked"` for the value mode `checked`).
    value_state: &'static str,
    /// The value-mode id that turns `value_state` on (defaults to the state's name).
    on_value: &'static str,
    indicator_only: Option<IndicatorOnly>,
    box_props: Option<BoxProps>,
    /// A variant used whatever the variant selector says (a combobox field is the `input` text field).
    fixed_variant: Option<&'static str>,
}

impl LssInspectorAdapter {
    pub fn new(control: &'static str, value_state: &'static str) -> Self {
        Self {
            control: ControlId::new(control),
            value_state,
            on_value: value_state,
            indicator_only: None,
            box_props: None,
            fixed_variant: None,
        }
    }

    /// Always inspect `variant`, ignoring the variant selector.
    pub fn fixed_variant(mut self, variant: &'static str) -> Self {
        self.fixed_variant = Some(variant);
        self
    }

    /// The value-mode id meaning on, when it differs from the state's name (e.g. `on` for `checked`).
    pub fn on_value(mut self, on_value: &'static str) -> Self {
        self.on_value = on_value;
        self
    }

    pub fn box_props(mut self, box_props: BoxProps) -> Self {
        self.box_props = Some(box_props);
        self
    }

    pub fn indicator_only(mut self, indicator_only: IndicatorOnly) -> Self {
        self.indicator_only = Some(indicator_only);
        self
    }

    pub fn shared(self) -> SharedInspectorResolver {
        Arc::new(self)
    }

    /// The style key for a selection. Variant and size ids that are not values of the control's axes
    /// (e.g. `primary`, `md`: the look's base) select the base.
    fn key(&self, theme: &Theme, selection: InspectorSelection<'_>) -> StyleKey {
        let sheet = theme.stylesheet();
        let config = sheet.config();
        let axes = sheet.axes(self.control);
        let mut key = StyleKey::new(self.control).scale(selection.scale_factor);
        for (axis, id) in [("variant", self.fixed_variant.unwrap_or(selection.variant_id)), ("size", selection.size_id)]
        {
            let Some(attr) = config.attribute(axis) else { continue };
            let name = id.replace('-', "_");
            let value = axes
                .attributes
                .iter()
                .find(|(a, _)| *a == attr)
                .and_then(|(_, values)| values.iter().find(|v| v.name() == name).copied());
            key = key.attr_opt(attr, value);
        }
        let s = interaction_state(selection.state_id);
        for (name, on) in [
            ("hover", s.hovered),
            ("pressed", s.pressed),
            ("focused", s.focused),
            ("disabled", s.disabled),
            (self.value_state, selection.value_id == self.on_value),
        ] {
            if let Some(state) = config.state(name) {
                key = key.state_if(state, on);
            }
        }
        key
    }

    fn inspect(&self, look: &ShadcnLook, selection: InspectorSelection<'_>) -> Inspection {
        let theme = look.style_theme();
        theme.inspect(&self.key(&theme, selection), None)
    }

    fn indicator_mode(&self, selection: InspectorSelection<'_>) -> Option<IndicatorOnly> {
        self.indicator_only.filter(|_| choice_indicator_only(selection.part_id))
    }

    /// The parts shown for a selection: the selected `.lss` part when the part selector names one (see
    /// [`lss_part_selector`]), otherwise every part; the indicator-only path hides its label.
    fn parts<'a>(&self, inspection: &'a Inspection, selection: InspectorSelection<'_>) -> Vec<&'a PartInspection> {
        if let Some(part) = selected_part(inspection, selection) {
            return vec![part];
        }
        let hidden = self.indicator_mode(selection).map_or(&[][..], |i| i.hidden_parts);
        inspection.parts.iter().filter(|p| !hidden.contains(&p.part.name())).collect()
    }

    fn colors(&self, inspection: &Inspection, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        self.parts(inspection, selection)
            .into_iter()
            .flat_map(|part| part.rows.iter().map(move |row| (part, row)))
            .filter(|(_, row)| row.category == Category::Colors)
            .filter_map(|(part, row)| match row.explanation.value {
                Some(Value::Color(color)) => Some(InspectColorRow {
                    label: row_label(part, row).into(),
                    value: Hsla::from(color),
                    source: row.explanation.source_line(),
                    detail: Some(provenance(row)),
                }),
                _ => None,
            })
            .collect()
    }

    fn property_rows(
        &self,
        inspection: &Inspection,
        selection: InspectorSelection<'_>,
        category: Category,
    ) -> Vec<InspectPropertyRow> {
        self.parts(inspection, selection)
            .into_iter()
            .flat_map(|part| part.rows.iter().map(move |row| (part, row)))
            .filter(|(_, row)| row.category == category && row.explanation.value.is_some())
            .map(|(part, row)| InspectPropertyRow {
                label: row_label(part, row),
                value: display_value(row.explanation.value.as_ref(), row.kind),
                source: row.explanation.source_line(),
                detail: Some(provenance(row)),
            })
            .collect()
    }

    fn shadows(&self, inspection: &Inspection, selection: InspectorSelection<'_>) -> Vec<gpui::BoxShadow> {
        self.parts(inspection, selection)
            .into_iter()
            .flat_map(|part| part.rows.iter())
            .filter_map(|row| match &row.explanation.value {
                Some(Value::Shadow(layers)) => Some(box_shadows(layers)),
                _ => None,
            })
            .flatten()
            .collect()
    }

    fn layout(
        &self,
        look: &ShadcnLook,
        inspection: &Inspection,
        selection: InspectorSelection<'_>,
    ) -> super::super::schema::InspectLayoutSection {
        // The selected part's box when the part selector names one, otherwise the root's.
        let root =
            selected_part(inspection, selection).or_else(|| inspection.parts.iter().find(|p| p.part == PartId::ROOT));
        let num = |name: &str| root.and_then(|r| number(r, name)).unwrap_or(0.0);
        let focus = FocusAdorner::new(FocusMetricTokens {
            gap: num("focus_gap"),
            width: num("focus_width"),
            offset: num("focus_offset"),
        });
        let box_model = match self.indicator_mode(selection) {
            Some(indicator) => InspectBoxModelSnapshot {
                height: num(indicator.size),
                padding_x: 0.0,
                padding_y: 0.0,
                border_width: num("border_width"),
                gap: 0.0,
                radius: num(indicator.radius),
                focus_ring_width: focus.width,
                focus_ring_offset: focus.gap,
            },
            None if self.box_props.is_some() => {
                let props = self.box_props.expect("checked");
                InspectBoxModelSnapshot {
                    height: num(props.height),
                    padding_x: num(props.padding),
                    padding_y: num(props.padding),
                    border_width: num("border_width"),
                    gap: num("gap"),
                    radius: num(props.radius),
                    focus_ring_width: focus.width,
                    focus_ring_offset: focus.gap,
                }
            }
            None => InspectBoxModelSnapshot {
                height: root.and_then(|r| r.box_model.height.or(r.box_model.min_height)).unwrap_or(0.0),
                padding_x: root.map_or(0.0, |r| r.box_model.padding.left),
                padding_y: root.map_or(0.0, |r| r.box_model.padding.top),
                border_width: root.map_or(0.0, |r| r.box_model.border_width),
                gap: num("gap"),
                radius: num("radius"),
                focus_ring_width: focus.width,
                focus_ring_offset: focus.gap,
            },
        };
        let disabled = interaction_state(selection.state_id).disabled;
        let shadows = self.shadows(inspection, selection);
        let occupation: InspectOccupationSnapshot =
            choice_indicator_occupation((!shadows.is_empty()).then_some(shadows.as_slice()), focus.extent(), disabled);
        let mut rows = self.property_rows(inspection, selection, Category::Layout);
        rows.extend(property_rows_from_metric_data(&occupation_metric_properties(box_model.height, &occupation)));
        let diagram = format!("{}-lss-box-model", self.control);
        layout_section(look, &diagram, box_model, Some(occupation), rows)
    }

    fn elevation(&self, inspection: &Inspection, selection: InspectorSelection<'_>) -> InspectElevationSnapshot {
        let shadows = self.shadows(inspection, selection);
        let catalog_value = self
            .parts(inspection, selection)
            .into_iter()
            .flat_map(|part| part.rows.iter())
            .filter(|row| row.category == Category::Elevation)
            .flat_map(|row| row.explanation.references.iter())
            .find(|r| r.kind == ReferenceKind::Native)
            .and_then(|r| r.description.clone());
        InspectElevationSnapshot {
            property_rows: self.property_rows(inspection, selection, Category::Elevation),
            catalog_value,
            layers: shadows
                .iter()
                .map(|s| InspectElevationLayer {
                    color: s.color,
                    display: format!(
                        "{}{} {} {} {}",
                        if s.inset { "inset " } else { "" },
                        f32::from(s.offset.x),
                        f32::from(s.offset.y),
                        f32::from(s.blur_radius),
                        f32::from(s.spread_radius)
                    ),
                })
                .collect(),
            preview_shadows: (!shadows.is_empty()).then_some(shadows),
        }
    }
}

impl ControlInspectorResolver for LssInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        let inspection = self.inspect(look, selection);
        match category_id {
            "layout" => InspectorCategoryContent::Layout(self.layout(look, &inspection, selection)),
            "elevation" => InspectorCategoryContent::Elevation(self.elevation(&inspection, selection)),
            "typography" => {
                InspectorCategoryContent::Typography(self.property_rows(&inspection, selection, Category::Typography))
            }
            _ => InspectorCategoryContent::Colors(self.colors(&inspection, selection)),
        }
    }

    fn category_applies(&self, look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
        match category_id {
            "elevation" => !self.shadows(&self.inspect(look, selection), selection).is_empty(),
            _ => true,
        }
    }
}

/// `indicator background` for a part, or the bare property for the root.
fn row_label(part: &PartInspection, row: &Row) -> String {
    let property = row.property.replace('_', " ");
    if part.part == PartId::ROOT {
        property
    } else {
        format!("{} {property}", part.part.name())
    }
}

/// The winning rule, what it beat, and every token it read, one per line.
fn provenance(row: &Row) -> String {
    let e = &row.explanation;
    std::iter::once(e.source_line())
        .chain(e.notes())
        .chain(e.reference_lines())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The part selector's `.lss` part: a part id written with `-` for `_` (e.g. `header`, `drag-handle`).
/// Template-path ids that name no part (`labeled`, `indicator-only`) and `all` select none.
fn selected_part<'a>(inspection: &'a Inspection, selection: InspectorSelection<'_>) -> Option<&'a PartInspection> {
    let name = selection.part_id.replace('-', "_");
    inspection.parts.iter().find(|p| p.part.name() == name && name != "all")
}

/// Several `.lss` controls under one spec: the part selector picks the control (by the part ids given),
/// the first one otherwise. A combobox is its text field and its menu.
pub struct LssCompositeAdapter {
    parts: Vec<(&'static str, LssInspectorAdapter)>,
}

impl LssCompositeAdapter {
    pub fn new(parts: Vec<(&'static str, LssInspectorAdapter)>) -> Self {
        Self { parts }
    }

    pub fn shared(self) -> SharedInspectorResolver {
        Arc::new(self)
    }

    fn pick(&self, selection: InspectorSelection<'_>) -> &LssInspectorAdapter {
        let found = self.parts.iter().find(|(id, _)| *id == selection.part_id);
        &found.unwrap_or(&self.parts[0]).1
    }
}

impl ControlInspectorResolver for LssCompositeAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        // The control's own rows: the part selector chose the control, not one of its parts.
        self.pick(selection)
            .resolve_category(look, InspectorSelection { part_id: "all", ..selection }, category_id)
    }

    fn category_applies(&self, look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
        self.pick(selection)
            .category_applies(look, InspectorSelection { part_id: "all", ..selection }, category_id)
    }
}

/// A part-selector entry for a `.lss` part (`id` is the part name, `-` for `_`; `all` shows every part).
pub const fn lss_part(id: &'static str, label: &'static str) -> InspectorPart {
    InspectorPart { id, label, variants: &[], default_variant_id: "" }
}

fn number(part: &PartInspection, name: &str) -> Option<f32> {
    part.rows
        .iter()
        .find(|r| r.property == name)
        .and_then(|r| r.explanation.value.as_ref())
        .and_then(Value::as_number)
}

static LSS_CATEGORIES: [InspectorCategory; 4] = [
    InspectorCategory { id: "color", label: "Color", icon: LucideIcon::Palette, expanded_default: true },
    InspectorCategory { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded_default: true },
    InspectorCategory { id: "elevation", label: "Elevation", icon: LucideIcon::Layers, expanded_default: true },
    InspectorCategory { id: "typography", label: "Typography", icon: LucideIcon::TypeIcon, expanded_default: true },
];

/// The SDK interaction states, each with every `.lss` category.
pub static LSS_INTERACTION_STATES: [InspectorStateSpec; 5] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &LSS_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &LSS_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &LSS_CATEGORIES,
    },
    InspectorStateSpec {
        id: "focused",
        label: "Focused",
        icon: LucideIcon::Focus,
        expanded_default: false,
        categories: &LSS_CATEGORIES,
    },
    InspectorStateSpec {
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &LSS_CATEGORIES,
    },
];

// The Shadcn checkbox (`look-shadcn/src/style/checkbox.lss`): primary is the base variant, md the base size.

static CHECKBOX_VARIANTS: [InspectorVariant; 5] = [
    InspectorVariant { id: "primary", label: "Primary" },
    InspectorVariant { id: "secondary", label: "Secondary" },
    InspectorVariant { id: "outline", label: "Outline" },
    InspectorVariant { id: "ghost", label: "Ghost" },
    InspectorVariant { id: "content-only", label: "Content Only" },
];

static CHECKBOX_PARTS: [InspectorPart; 2] = [
    InspectorPart { id: "labeled", label: "Labeled", variants: &CHECKBOX_VARIANTS, default_variant_id: "primary" },
    InspectorPart {
        id: "indicator-only",
        label: "Indicator Only",
        variants: &CHECKBOX_VARIANTS,
        default_variant_id: "primary",
    },
];

static CHECKBOX_SIZES: [InspectorSize; 3] = [
    InspectorSize { id: "sm", label: "sm" },
    InspectorSize { id: "md", label: "md" },
    InspectorSize { id: "lg", label: "lg" },
];

static CHECKBOX_VALUES: [InspectorValueMode; 2] = [
    InspectorValueMode { id: "checked", label: "checked" },
    InspectorValueMode { id: "unchecked", label: "unchecked" },
];

pub static CHECKBOX_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Checkbox",
    id_prefix: "checkbox-theme-inspector",
    parts: &CHECKBOX_PARTS,
    variants: &[],
    states: &LSS_INTERACTION_STATES,
    sizes: &CHECKBOX_SIZES,
    value_modes: &CHECKBOX_VALUES,
    default_part_id: "labeled",
    default_variant_id: "primary",
    default_size_id: "md",
    default_value_id: "checked",
};

static RADIO_VALUES: [InspectorValueMode; 2] = [
    InspectorValueMode { id: "selected", label: "selected" },
    InspectorValueMode { id: "unselected", label: "unselected" },
];

/// The Shadcn radio (`look-shadcn/src/style/radio.lss`): the same parts, variants and sizes as the checkbox.
pub static RADIO_BUTTON_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Radio Button",
    id_prefix: "radio-button-theme-inspector",
    parts: &CHECKBOX_PARTS,
    variants: &[],
    states: &LSS_INTERACTION_STATES,
    sizes: &CHECKBOX_SIZES,
    value_modes: &RADIO_VALUES,
    default_part_id: "labeled",
    default_variant_id: "primary",
    default_size_id: "md",
    default_value_id: "selected",
};

static SWITCH_VALUES: [InspectorValueMode; 2] =
    [InspectorValueMode { id: "on", label: "on" }, InspectorValueMode { id: "off", label: "off" }];

/// The Shadcn switch (`look-shadcn/src/style/switch.lss`): no parts selector; the box model is the track.
pub static SWITCH_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Switch",
    id_prefix: "switch-theme-inspector",
    parts: &[],
    variants: &CHECKBOX_VARIANTS,
    states: &LSS_INTERACTION_STATES,
    sizes: &CHECKBOX_SIZES,
    value_modes: &SWITCH_VALUES,
    default_part_id: "",
    default_variant_id: "primary",
    default_size_id: "md",
    default_value_id: "on",
};

/// The switch inspector: the generic adapter with the track as the box model.
pub fn switch_inspector() -> SharedInspectorResolver {
    LssInspectorAdapter::new("switch", "checked")
        .on_value("on")
        .box_props(BoxProps { height: "track_height", padding: "track_padding", radius: "track_radius" })
        .shared()
}

/// The radio inspector: the generic adapter with the radio's indicator geometry.
pub fn radio_button_inspector() -> SharedInspectorResolver {
    LssInspectorAdapter::new("radio", "selected")
        .indicator_only(IndicatorOnly { size: "indicator_size", radius: "indicator_radius", hidden_parts: &["label"] })
        .shared()
}

/// The checkbox inspector: the generic adapter with the checkbox's indicator geometry.
pub fn checkbox_inspector() -> SharedInspectorResolver {
    LssInspectorAdapter::new("checkbox", "checked")
        .indicator_only(IndicatorOnly { size: "indicator_size", radius: "indicator_radius", hidden_parts: &["label"] })
        .shared()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection<'a>(variant: &'a str, state: &'a str, value: &'a str, part: &'a str) -> InspectorSelection<'a> {
        InspectorSelection {
            part_id: part,
            variant_id: variant,
            state_id: state,
            size_id: "md",
            value_id: value,
            scale_factor: 1.0,
        }
    }

    #[test]
    fn checkbox_rows_come_from_lss_with_provenance() {
        let look = ShadcnLook::from_css_str(xgpui_luma_look_shadcn::FALLBACK_CSS).expect("catalog");
        let adapter = checkbox_inspector();
        let InspectorCategoryContent::Colors(colors) =
            adapter.resolve_category(&look, selection("primary", "default", "checked", "labeled"), "color")
        else {
            panic!("colors")
        };
        let background = colors.iter().find(|r| r.label.as_ref() == "indicator background").expect("background row");
        assert!(background.source.contains("checkbox.lss"), "{}", background.source);
        assert!(background.detail.as_deref().is_some_and(|d| d.contains("action_layer")), "{:?}", background.detail);
        assert!(colors.iter().any(|r| r.label.as_ref() == "label foreground"));

        let InspectorCategoryContent::Colors(indicator_only) =
            adapter.resolve_category(&look, selection("primary", "default", "checked", "indicator-only"), "color")
        else {
            panic!("colors")
        };
        assert!(indicator_only.iter().all(|r| !r.label.starts_with("label")), "the label is hidden");
    }

    #[test]
    fn checkbox_layout_has_box_model_occupation_and_native_provenance() {
        let look = ShadcnLook::from_css_str(xgpui_luma_look_shadcn::FALLBACK_CSS).expect("catalog");
        let adapter = checkbox_inspector();
        let InspectorCategoryContent::Layout(layout) =
            adapter.resolve_category(&look, selection("primary", "default", "unchecked", "labeled"), "layout")
        else {
            panic!("layout")
        };
        assert_eq!(layout.box_model.height, 36.0);
        assert_eq!(layout.box_model.gap, 8.0);
        assert!(layout.occupation.is_some_and(|o| o.has_overflow()), "shadow and focus ring paint outside the box");
        let radius = layout.metrics.iter().find(|r| r.label == "radius").expect("radius row");
        assert!(radius.detail.as_deref().is_some_and(|d| d.contains("--radius")), "{:?}", radius.detail);

        let InspectorCategoryContent::Layout(indicator) =
            adapter.resolve_category(&look, selection("primary", "default", "unchecked", "indicator-only"), "layout")
        else {
            panic!("layout")
        };
        assert_eq!(indicator.box_model.height, 18.0);
    }

    #[test]
    fn elevation_applies_only_where_the_lss_paints_a_shadow() {
        let look = ShadcnLook::from_css_str(xgpui_luma_look_shadcn::FALLBACK_CSS).expect("catalog");
        let adapter = checkbox_inspector();
        assert!(adapter.category_applies(&look, selection("primary", "default", "unchecked", "labeled"), "elevation"));
        assert!(!adapter.category_applies(
            &look,
            selection("primary", "disabled", "unchecked", "labeled"),
            "elevation"
        ));
        assert!(!adapter.category_applies(
            &look,
            selection("content-only", "default", "unchecked", "labeled"),
            "elevation"
        ));
        let InspectorCategoryContent::Elevation(elevation) =
            adapter.resolve_category(&look, selection("primary", "default", "unchecked", "labeled"), "elevation")
        else {
            panic!("elevation")
        };
        assert_eq!(elevation.layers.len(), 1);
        assert!(
            elevation.catalog_value.as_deref().is_some_and(|d| d.contains("--shadow-xs")),
            "{:?}",
            elevation.catalog_value
        );
    }
}

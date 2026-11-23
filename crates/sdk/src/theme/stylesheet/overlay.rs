//! Layering for authored stylesheet TOML: later layers overlay earlier ones field by field.

/// Tables inherit recursively; values and arrays replace their base, including
/// zero, false and empty arrays. Motion transitions and tagged source colors
/// replace complete values rather than inheriting their individual components.
pub(super) fn overlay(base: &mut toml::Value, authored: toml::Value) {
    if base.get("kind").is_some()
        || authored.get("kind").is_some()
        || base.get("space").is_some()
        || authored.get("space").is_some()
    {
        *base = authored;
        return;
    }
    match (base, authored) {
        (toml::Value::Table(base), toml::Value::Table(authored)) => {
            // A later layer's general geometry outranks earlier size/variant
            // values. Specific values authored in this layer still apply last.
            if let Some(toml::Value::Table(geometry)) = authored.get("geometry") {
                for axis in ["sizes", "variants"] {
                    if let Some(toml::Value::Table(variants)) = base.get_mut(axis) {
                        for (_, variant) in variants.iter_mut() {
                            if let toml::Value::Table(variant) = variant {
                                for key in geometry.keys() {
                                    variant.remove(key);
                                }
                            }
                        }
                    }
                }
            }
            for (key, value) in authored {
                if let Some(existing) = base.get_mut(&key) {
                    overlay(existing, value);
                } else {
                    base.insert(key, value);
                }
            }
        }
        (base, authored) => *base = authored,
    }
}

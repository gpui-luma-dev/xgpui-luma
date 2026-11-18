use std::collections::BTreeSet;
use super::{Catalog, CatalogError};

fn identifier(value: &str) -> String {
    value.replace(['-', '.'], "_").to_ascii_lowercase()
}

impl Catalog {
    /// Generate deterministic Rust constructors. Each requires exactly the
    /// English message's caller arguments. Identifier collisions are errors.
    ///
    /// The generated file uses `::xgpui_luma_l10n` and should be included inside
    /// a module. Fluent attributes are supported; private terms have no constructor.
    pub fn generate_rust(&self) -> Result<String, CatalogError> {
        let mut output = String::from("// Generated from Fluent; do not edit.\n");
        output.push_str("#[allow(dead_code)]\npub const MESSAGE_IDS: &[&str] = &[\n");
        for id in self.messages.keys() {
            output.push_str(&format!("    {id:?},\n"));
        }
        output.push_str("];\n");
        let mut names = BTreeSet::new();
        for (id, arguments) in &self.messages {
            let name = format!("msg_{}", identifier(id));
            if !names.insert(name.clone()) {
                return Err(CatalogError(format!("generated Rust name collision at {id}: {name}")));
            }
            let params: Vec<_> = arguments.iter().map(|arg| format!("arg_{}", identifier(arg))).collect();
            if params.iter().collect::<BTreeSet<_>>().len() != params.len() {
                return Err(CatalogError(format!("generated Rust argument name collision at {id}")));
            }
            output.push_str(&format!(
                "/// Fluent `{id}`; arguments in order: {:?}.\n",
                arguments.iter().collect::<Vec<_>>()
            ));
            output.push_str(&format!("#[allow(dead_code)]\npub fn {name}<'a>("));
            let signature: Vec<_> = params
                .iter()
                .map(|param| format!("{param}: impl Into<::xgpui_luma_l10n::FluentValue<'a>>"))
                .collect();
            output.push_str(&signature.join(", "));
            output.push_str(") -> ::xgpui_luma_l10n::Message<'a> {\n");
            output.push_str(if arguments.is_empty() {
                "let args = "
            } else {
                "let mut args = "
            });
            output.push_str("::xgpui_luma_l10n::FluentArgs::new();\n");
            for (arg, param) in arguments.iter().zip(&params) {
                output.push_str(&format!("args.set({arg:?}, {param});\n"));
            }
            let (message, attribute) = id
                .split_once('.')
                .map_or((id.as_str(), "None".into()), |(message, attr)| (message, format!("Some({attr:?})")));
            output.push_str(&format!("::xgpui_luma_l10n::Message::new({message:?}, {attribute}, args)\n}}\n"));
        }
        Ok(output)
    }
}

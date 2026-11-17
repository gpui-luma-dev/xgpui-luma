use fluent_syntax::ast::{Entry, Pattern, PatternElement, Expression, InlineExpression};
fn pattern(value: &mut Pattern<String>, rtl: bool) {
    for element in &mut value.elements {
        match element {
            PatternElement::TextElement { value } => {
                if value.trim().is_empty() {
                    continue;
                }
                let text: String = value
                    .chars()
                    .map(|c| match c {
                        'a' => 'á',
                        'e' => 'é',
                        'i' => 'î',
                        'o' => 'ö',
                        'u' => 'ü',
                        'A' => 'Á',
                        'E' => 'É',
                        'I' => 'Î',
                        'O' => 'Ö',
                        'U' => 'Ü',
                        _ => c,
                    })
                    .collect();
                let pad = "~".repeat((value.chars().count() * 2).div_ceil(5));
                *value = if rtl {
                    format!("\u{200f}⟦{text} {pad}⟧\u{200f}")
                } else {
                    format!("⟦{text} {pad}⟧")
                };
            }
            PatternElement::Placeable { expression } => expression_pattern(expression, rtl),
        }
    }
}
fn expression_pattern(expression: &mut Expression<String>, rtl: bool) {
    match expression {
        Expression::Select { variants, .. } => {
            for variant in variants {
                pattern(&mut variant.value, rtl);
            }
        }
        Expression::Inline(InlineExpression::Placeable { expression }) => expression_pattern(expression, rtl),
        _ => {}
    }
}
pub fn resource(source: &str, rtl: bool) -> Result<String, String> {
    let mut parsed = fluent_syntax::parser::parse(source.to_owned()).map_err(|(_, errors)| format!("{errors:?}"))?;
    for entry in &mut parsed.body {
        match entry {
            Entry::Message(message) => {
                if let Some(value) = &mut message.value {
                    pattern(value, rtl);
                }
                for attribute in &mut message.attributes {
                    pattern(&mut attribute.value, rtl);
                }
            }
            Entry::Term(term) => {
                pattern(&mut term.value, rtl);
                for attribute in &mut term.attributes {
                    pattern(&mut attribute.value, rtl);
                }
            }
            _ => {}
        }
    }
    Ok(fluent_syntax::serializer::serialize(&parsed))
}

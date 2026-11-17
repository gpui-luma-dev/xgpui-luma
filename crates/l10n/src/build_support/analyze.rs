use fluent_syntax::ast;
use super::Node;

pub(super) fn pattern(pattern: &ast::Pattern<&str>, node: &mut Node) {
    for element in &pattern.elements {
        if let ast::PatternElement::Placeable { expression: value } = element {
            expression(value, node);
        }
    }
}

fn expression(value: &ast::Expression<&str>, node: &mut Node) {
    match value {
        ast::Expression::Inline(value) => inline(value, node),
        ast::Expression::Select { selector, variants } => {
            inline(selector, node);
            for variant in variants {
                pattern(&variant.value, node);
            }
        }
    }
}

fn arguments(args: &ast::CallArguments<&str>, node: &mut Node) {
    for value in &args.positional {
        inline(value, node);
    }
    for value in &args.named {
        inline(&value.value, node);
    }
}

fn inline(value: &ast::InlineExpression<&str>, node: &mut Node) {
    match value {
        ast::InlineExpression::VariableReference { id } => {
            node.variables.insert(id.name.to_owned());
        }
        ast::InlineExpression::MessageReference { id, attribute } => {
            let key =
                attribute.as_ref().map_or_else(|| id.name.to_owned(), |attr| format!("{}.{}", id.name, attr.name));
            node.references.push((key, true));
        }
        ast::InlineExpression::TermReference { id, attribute, arguments: args } => {
            let key = attribute
                .as_ref()
                .map_or_else(|| format!("-{}", id.name), |attr| format!("-{}.{}", id.name, attr.name));
            node.references.push((key, false));
            if let Some(args) = args {
                arguments(args, node);
            }
        }
        ast::InlineExpression::FunctionReference { arguments: args, .. } => arguments(args, node),
        ast::InlineExpression::Placeable { expression: value } => expression(value, node),
        ast::InlineExpression::StringLiteral { .. } | ast::InlineExpression::NumberLiteral { .. } => {}
    }
}

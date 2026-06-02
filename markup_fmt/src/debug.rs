//! Debugging helpers for visualizing the formatter's intermediate
//! representation (e.g. the playground "Doc IR" view).

use crate::{
    config::FormatOptions, ctx::Ctx, ctx::Hints, error::FormatError, parser::Language,
    parser::Parser, printer::DocGen, state::State,
};
use std::borrow::Cow;
use tiny_pretty::Doc;

/// Build the intermediate [`tiny_pretty::Doc`] tree for the given source and
/// render it as a human-readable, indented tree.
///
/// This is the same `Doc` IR that [`crate::format_text`] feeds to the pretty
/// printer, exposed for debugging and visualization.
pub fn debug_doc_tree<E, F>(
    code: &str,
    language: Language,
    options: &FormatOptions,
    external_formatter: F,
) -> Result<String, FormatError<E>>
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    let mut parser = Parser::new(
        code,
        language,
        options.language.custom_blocks.clone().unwrap_or_default(),
    );
    let ast = parser.parse_root().map_err(FormatError::Syntax)?;

    let mut ctx = Ctx {
        source: code,
        language,
        indent_width: options.layout.indent_width,
        print_width: options.layout.print_width,
        options: &options.language,
        external_formatter,
        external_formatter_errors: Default::default(),
    };

    let doc = ast.doc(
        &mut ctx,
        &State {
            current_tag_name: None,
            is_root: true,
            in_svg: false,
            indent_level: 0,
        },
    );
    if !ctx.external_formatter_errors.is_empty() {
        return Err(FormatError::External(ctx.external_formatter_errors));
    }

    let mut out = String::new();
    write_doc(&mut out, &doc, 0);
    Ok(out)
}

/// Recursively render a [`tiny_pretty::Doc`] node as an indented tree line.
fn write_doc(out: &mut String, doc: &Doc, indent: usize) {
    use std::fmt::Write;

    let prefix = "│  ".repeat(indent);
    match doc {
        Doc::Nil => {
            let _ = writeln!(out, "{prefix}Nil");
        }
        Doc::NewLine => {
            let _ = writeln!(out, "{prefix}NewLine");
        }
        Doc::EmptyLine => {
            let _ = writeln!(out, "{prefix}EmptyLine");
        }
        Doc::Text(text) => {
            let _ = writeln!(out, "{prefix}Text({text:?})");
        }
        Doc::Break(size, offset) => {
            let _ = writeln!(out, "{prefix}Break(size={size}, offset={offset})");
        }
        Doc::Nest(width, inner) => {
            let _ = writeln!(out, "{prefix}Nest({width})");
            write_doc(out, inner, indent + 1);
        }
        Doc::Alt(flat, broken) => {
            let _ = writeln!(out, "{prefix}Alt");
            write_doc(out, flat, indent + 1);
            write_doc(out, broken, indent + 1);
        }
        Doc::Union(first, second) => {
            let _ = writeln!(out, "{prefix}Union");
            write_doc(out, first, indent + 1);
            write_doc(out, second, indent + 1);
        }
        Doc::Group(docs) => {
            let _ = writeln!(out, "{prefix}Group");
            for child in docs {
                write_doc(out, child, indent + 1);
            }
        }
        Doc::List(docs) => {
            let _ = writeln!(out, "{prefix}List");
            for child in docs {
                write_doc(out, child, indent + 1);
            }
        }
    }
}

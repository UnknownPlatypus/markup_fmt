//! Debugging helpers for visualizing the formatter's intermediate
//! representation (e.g. the playground "Doc IR" view).

use crate::{
    config::FormatOptions, ctx::Ctx, ctx::Hints, error::FormatError, parser::Language,
    parser::Parser, printer::DocGen, state::State,
};
use anyhow::Error;
use std::borrow::Cow;

/// Build the intermediate [`tiny_pretty::Doc`] tree for the given source and
/// render it as a pretty-printed, indented tree.
///
/// This is the same `Doc` IR that [`crate::format_text`] feeds to the pretty
/// printer, exposed for debugging and visualization. The tree is rendered with
/// `Doc`'s `Debug` representation because `tiny_pretty` does not expose its
/// `Nest` wrapper type, so a hand-rolled walker cannot descend into nested docs.
pub fn debug_doc_tree<F>(
    code: &str,
    language: Language,
    options: &FormatOptions,
    external_formatter: F,
) -> Result<String, FormatError>
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, Error>,
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
            in_attr_loop: false,
        },
    );
    if !ctx.external_formatter_errors.is_empty() {
        return Err(FormatError::External(ctx.external_formatter_errors));
    }

    Ok(format!("{doc:#?}"))
}

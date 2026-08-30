#![doc = include_str!("../README.md")]

pub mod ast;
pub mod config;
mod ctx;
pub mod debug;
mod error;
mod helpers;
pub mod parser;
mod printer;
mod state;

use crate::{config::FormatOptions, ctx::Ctx, parser::Parser, printer::DocGen, state::State};
pub use crate::{
    ctx::Hints, debug::debug_doc_tree, error::*, helpers::starts_with_directive, parser::Language,
};
use anyhow::Error;
use std::{borrow::Cow, path::Path};
use tiny_pretty::{IndentKind, PrintOptions};

/// Format the given source code.
///
/// An external formatter is required for formatting code
/// inside `<script>` or `<style>` tag.
/// If you don't need to format them or you don't have available formatters,
/// you can pass a closure that returns the original code. (see example below)
///
/// ```
/// use markup_fmt::{format_text, Language};
///
/// let code = r#"
/// <html>
///    <head>
///      <title>Example</title>
///      <style>button { outline: none; }</style>
///   </head>
///   <body><script>const a = 1;</script></body>
/// </html>"#;
///
/// let formatted = format_text(
///     code,
///     Language::Html,
///     &Default::default(),
///     |code, _| Ok(code.into()),
/// ).unwrap();
/// ```
///
/// For the external formatter closure,
///
/// - The first argument is code that needs formatting.
/// - The second argument is hints which contains useful information for external formatters,
///   such as file extension and print width.
pub fn format_text<F>(
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

    if ast.children.first().is_some_and(|child| {
        if let ast::Node {
            kind:
                ast::NodeKind::Comment(ast::Comment { raw, .. })
                | ast::NodeKind::JinjaComment(ast::JinjaComment { raw, .. }),
            ..
        } = child
        {
            options
                .language
                .ignore_file_comment_directive
                .iter()
                .any(|directive| starts_with_directive(raw, directive))
        } else {
            false
        }
    }) {
        return Ok(code.into());
    }

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

    Ok(tiny_pretty::print(
        &doc,
        &PrintOptions {
            indent_kind: if options.layout.use_tabs {
                IndentKind::Tab
            } else {
                IndentKind::Space
            },
            line_break: options.layout.line_break.into(),
            width: options.layout.print_width,
            tab_size: options.layout.indent_width,
        },
    ))
}

/// Detect language from file extension.
pub fn detect_language(path: impl AsRef<Path>) -> Option<Language> {
    let path = path.as_ref();
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("html") => {
            if path
                .file_stem()
                .is_some_and(|file_stem| file_stem.to_string_lossy().ends_with(".component"))
            {
                Some(Language::Angular)
            } else {
                Some(Language::Html)
            }
        }
        Some("vue") => Some(Language::Vue),
        Some("svelte") => Some(Language::Svelte),
        Some("astro") => Some(Language::Astro),
        Some("jinja" | "jinja2" | "j2" | "twig" | "njk") => Some(Language::Jinja),
        Some("vto") => Some(Language::Vento),
        Some("mustache" | "hbs" | "handlebars") => Some(Language::Mustache),
        Some("xml" | "svg" | "wsdl" | "xsd" | "xslt" | "xsl") => Some(Language::Xml),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mjs() {
        let mut ext = None;
        let _ = format_text(
            "<script type=module>;</script>",
            Language::Html,
            &Default::default(),
            |code, hints| {
                ext = Some(hints.ext.to_owned());
                Ok(Cow::from(code))
            },
        );
        assert_eq!(ext.as_deref(), Some("mjs"));
    }

    #[test]
    fn mts() {
        let mut ext = None;
        let _ = format_text(
            "<script type=\"module\" lang='ts'>;</script>",
            Language::Html,
            &Default::default(),
            |code, hints| {
                ext = Some(hints.ext.to_owned());
                Ok(Cow::from(code))
            },
        );
        assert_eq!(ext.as_deref(), Some("mts"));
    }

    #[test]
    fn jsx_with_module() {
        let mut ext = None;
        let _ = format_text(
            "<script type=module lang=jsx>;</script>",
            Language::Html,
            &Default::default(),
            |code, hints| {
                ext = Some(hints.ext.to_owned());
                Ok(Cow::from(code))
            },
        );
        assert_eq!(ext.as_deref(), Some("jsx"));
    }

    #[test]
    fn tsx_with_module() {
        let mut ext = None;
        let _ = format_text(
            "<script type=\"module\" lang='tsx'>;</script>",
            Language::Html,
            &Default::default(),
            |code, hints| {
                ext = Some(hints.ext.to_owned());
                Ok(Cow::from(code))
            },
        );
        assert_eq!(ext.as_deref(), Some("tsx"));
    }

    #[test]
    fn unterminated_interpolation_in_tag_is_rejected() {
        for input in [
            "<h{{ level }",
            "<input {{ field.attrs }",
            "<div data-{{ key }",
            "<option value={{ id }",
            "<div class=btn-{{ variant }",
        ] {
            let err = format_text(input, Language::Jinja, &Default::default(), |code, _| {
                Ok(Cow::from(code))
            })
            .unwrap_err();
            assert!(
                matches!(
                    err,
                    FormatError::Syntax(SyntaxError {
                        kind: SyntaxErrorKind::ExpectAttrName,
                        ..
                    })
                ),
                "expected an attribute name error for {input:?}, got {err}"
            );
        }
    }

    #[test]
    fn django_whitespace_control_is_rejected() {
        for input in [
            "{% for item in seq -%}{{ item }}{% endfor %}",
            "{%+ if bar %}yes{% endif %}",
            "{{- foo }}",
            "{{ foo -}}",
            "{% comment %}x{%- endcomment %}",
        ] {
            let err = format_text(input, Language::Django, &Default::default(), |code, _| {
                Ok(Cow::from(code))
            })
            .unwrap_err();
            assert!(
                matches!(
                    err,
                    FormatError::Syntax(SyntaxError {
                        kind: SyntaxErrorKind::DjangoWhitespaceControl,
                        ..
                    })
                ),
                "expected a whitespace control error for {input:?}, got {err}"
            );
        }
    }

    #[test]
    fn unterminated_jinja_tag_is_rejected() {
        for language in [Language::Jinja, Language::Django] {
            // Pre-fix, all of these formatted with the tag content replaced by `{%  %}`.
            for input in [
                "{%",
                "{% if x",
                "{% cycle 'a' 'b'",
                "{% if x %}body{% endif %}{% cycle 'a'",
            ] {
                let err = format_text(input, language, &Default::default(), |code, _| {
                    Ok(Cow::from(code))
                })
                .unwrap_err();
                assert!(
                    matches!(
                        err,
                        FormatError::Syntax(SyntaxError {
                            kind: SyntaxErrorKind::ExpectChar('}'),
                            ..
                        })
                    ),
                    "expected a missing `%}}` error for {input:?} in {language:?}, got {err}"
                );
            }
        }
    }

    #[test]
    fn unclosed_django_raw_block_is_rejected() {
        for input in [
            "{%comment%}",
            "{% comment %}\n",
            "<p>{% comment %}hi",
            "{% verbatim %}",
            "{% verbatim x %}raw",
            "{% comment %}x{% endcommentary %}",
            "{% verbatim %}x{% endverbatim y %}",
        ] {
            let err = format_text(input, Language::Django, &Default::default(), |code, _| {
                Ok(Cow::from(code))
            })
            .unwrap_err();
            assert!(
                matches!(
                    err,
                    FormatError::Syntax(SyntaxError {
                        kind: SyntaxErrorKind::ExpectJinjaBlockEnd { .. },
                        ..
                    })
                ),
                "expected a missing end tag error for {input:?}, got {err}"
            );
        }
    }
}

use insta::{Settings, assert_snapshot, glob};
use markup_fmt::{
    Hints, Language,
    config::{FormatOptions, ScriptFormatter},
    detect_language, format_text,
};
use std::{borrow::Cow, collections::HashMap, fs, path::Path};

#[test]
fn fmt_snapshot() {
    let pattern = "fmt/**/*.{html,vue,svelte,astro,jinja,njk,vto,mustache,hbs,xml}";
    glob!(pattern, |path| {
        let input = fs::read_to_string(path).unwrap();
        // Match the `django` fixture dir as a path component, not a substring,
        // so a checkout path containing "django" doesn't hijack every fixture.
        let language = if path.components().any(|c| c.as_os_str() == "django") {
            Language::Django
        } else {
            detect_language(path).unwrap()
        };

        let options = fs::read_to_string(path.with_file_name("config.toml"))
            .map(|config_file| {
                toml::from_str::<HashMap<String, FormatOptions>>(&config_file).unwrap()
            })
            .ok();

        if let Some(options) = options {
            options.into_iter().for_each(|(option_name, options)| {
                let output = run_format_test(path, &input, &options, language);
                build_settings(path).bind(|| {
                    let name = path.file_stem().unwrap().to_str().unwrap();
                    assert_snapshot!(format!("{name}.{option_name}"), output);
                });
            })
        } else {
            let output = run_format_test(path, &input, &Default::default(), language);
            build_settings(path).bind(|| {
                let name = path.file_stem().unwrap().to_str().unwrap();
                assert_snapshot!(name, output);
            });
        }
    });
}

/// Unterminated interpolations used to be auto-closed with a `}}` that the next parse
/// couldn't match back, so every pass appended one more `}`.
/// Angular recovers into a text node instead, covered by `angular/interpolation/unterminated`.
#[test]
fn unterminated_interpolation_is_a_syntax_error() {
    let format = |input: &str, language| {
        format_text(input, language, &Default::default(), |code, _| {
            Ok(code.into())
        })
    };

    for input in ["{{{", "{{\"", "{{'", "{{`", "{{ a", "{{ \"}}"] {
        for language in [
            Language::Vue,
            Language::Svelte,
            Language::Jinja,
            Language::Vento,
            Language::Mustache,
        ] {
            assert!(
                format(input, language).is_err(),
                "{input:?} should be a syntax error in {language:?}"
            );
        }
    }
}

fn run_format_test(
    path: &Path,
    input: &str,
    options: &FormatOptions,
    language: Language,
) -> String {
    let output = format_text(input, language, options, |code, hints| {
        Ok(mock_external_formatter(code, &hints, options))
    })
    .map_err(|err| format!("failed to format '{}': {:?}", path.display(), err))
    .unwrap();
    let regression_format = format_text(&output, language, options, |code, hints| {
        Ok(mock_external_formatter(code, &hints, options))
    })
    .map_err(|err| {
        format!(
            "syntax error in stability test '{}': {err:?}",
            path.display(),
        )
    })
    .unwrap();
    similar_asserts::assert_eq!(
        output,
        regression_format,
        "'{}' format is unstable",
        path.display()
    );

    output
}

/// Emulate dprint's `file_indent_level` handling when a fixture opts into
/// `script_formatter = "dprint"`: dedent the script, then indent every line
/// to `hints.indent_level`. Other fixtures keep the identity formatter.
fn mock_external_formatter<'a>(
    code: &'a str,
    hints: &Hints,
    options: &FormatOptions,
) -> Cow<'a, str> {
    if !matches!(
        options.language.script_formatter,
        Some(ScriptFormatter::Dprint)
    ) || !matches!(hints.ext, "js" | "mjs" | "jsx" | "ts" | "mts" | "tsx")
    {
        return code.into();
    }
    let base = " ".repeat(usize::from(hints.indent_level) * options.layout.indent_width);
    let dedent = code
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    code.lines()
        .map(|line| {
            let line = line.trim_end();
            if line.is_empty() {
                String::new()
            } else {
                format!("{base}{}", &line[dedent..])
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .into()
}

fn build_settings(path: &Path) -> Settings {
    let mut settings = Settings::clone_current();
    settings.set_snapshot_path(path.parent().unwrap());
    settings.remove_snapshot_suffix();
    settings.set_prepend_module_to_snapshot(false);
    settings.remove_input_file();
    settings.set_omit_expression(true);
    settings.remove_input_file();
    settings.remove_info();
    settings
}

/// Directive lookups read `Root::jinja_comments` instead of walking the tree.
#[test]
fn root_indexes_jinja_comment_bodies() {
    let source = "{# a #}<div {#- b -#} class=\"x\">{% if c %}{# d #}{% endif %}</div>";
    let root = markup_fmt::parser::Parser::new(source, Language::Jinja, vec![])
        .parse_root()
        .unwrap();
    assert_eq!(root.jinja_comments, [" a ", "- b -", " d "]);
}

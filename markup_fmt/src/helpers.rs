use crate::Language;
use aho_corasick::AhoCorasick;
use std::{borrow::Cow, cmp::Ordering, fmt, ops::ControlFlow, sync::LazyLock};

pub(crate) fn is_component(name: &str) -> bool {
    name.contains('-') || name.contains(|c: char| c.is_ascii_uppercase())
}

static NON_WS_SENSITIVE_TAGS: [&str; 76] = [
    "address",
    "blockquote",
    "button",
    "caption",
    "center",
    "colgroup",
    "dialog",
    "div",
    "figure",
    "figcaption",
    "footer",
    "form",
    "select",
    "option",
    "optgroup",
    "header",
    "hr",
    "legend",
    "listing",
    "main",
    "p",
    "plaintext",
    "pre",
    "progress",
    "search",
    "object",
    "details",
    "summary",
    "xmp",
    "area",
    "base",
    "basefont",
    "datalist",
    "head",
    "link",
    "meta",
    "meter",
    "noembed",
    "noframes",
    "param",
    "rp",
    "title",
    "html",
    "body",
    "article",
    "aside",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hgroup",
    "nav",
    "section",
    "table",
    "tr",
    "thead",
    "th",
    "tbody",
    "td",
    "tfoot",
    "dir",
    "dd",
    "dl",
    "dt",
    "menu",
    "ol",
    "ul",
    "li",
    "fieldset",
    "video",
    "audio",
    "picture",
    "source",
    "track",
];

pub(crate) fn is_whitespace_sensitive_tag(name: &str, language: Language) -> bool {
    match language {
        Language::Html
        | Language::Jinja
        | Language::Django
        | Language::Vento
        | Language::Mustache => {
            // There's also a tag called "a" in SVG, so we need to check it specially.
            name.eq_ignore_ascii_case("a")
                || !NON_WS_SENSITIVE_TAGS
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(name))
                    && !css_dataset::tags::SVG_TAGS
                        .iter()
                        .any(|tag| tag.eq_ignore_ascii_case(name))
        }
        Language::Xml => false,
        _ => {
            name == "a"
                || !NON_WS_SENSITIVE_TAGS.contains(&name)
                    && !css_dataset::tags::SVG_TAGS.contains(&name)
        }
    }
}

static VOID_ELEMENTS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr", "param",
];

pub(crate) fn is_void_element(name: &str, language: Language) -> bool {
    match language {
        Language::Html
        | Language::Jinja
        | Language::Django
        | Language::Vento
        | Language::Mustache => VOID_ELEMENTS
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(name)),
        Language::Xml => false,
        _ => VOID_ELEMENTS.contains(&name),
    }
}

pub(crate) fn is_html_tag(name: &str, language: Language) -> bool {
    match language {
        Language::Html
        | Language::Jinja
        | Language::Django
        | Language::Vento
        | Language::Mustache => {
            css_dataset::tags::STANDARD_HTML_TAGS
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(name))
                || css_dataset::tags::NON_STANDARD_HTML_TAGS
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(name))
                || css_dataset::tags::NON_STANDARD_HTML_TAGS
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(name))
        }
        Language::Xml => false,
        _ => {
            css_dataset::tags::STANDARD_HTML_TAGS.contains(&name)
                || css_dataset::tags::NON_STANDARD_HTML_TAGS.contains(&name)
        }
    }
}

pub(crate) fn is_svg_tag(name: &str, language: Language) -> bool {
    if matches!(
        language,
        Language::Html | Language::Jinja | Language::Django | Language::Vento | Language::Mustache
    ) {
        css_dataset::tags::SVG_TAGS
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(name))
    } else {
        css_dataset::tags::SVG_TAGS.contains(&name)
    }
}

pub(crate) fn is_mathml_tag(name: &str, language: Language) -> bool {
    match language {
        Language::Html
        | Language::Jinja
        | Language::Django
        | Language::Vento
        | Language::Mustache => css_dataset::tags::MATH_ML_TAGS
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(name)),
        Language::Xml => false,
        _ => css_dataset::tags::MATH_ML_TAGS.contains(&name),
    }
}

pub(crate) fn parse_vento_tag(tag: &str) -> (&str, &str) {
    let trimmed = tag.trim();
    trimmed
        .split_once(|c: char| c.is_ascii_whitespace())
        .unwrap_or((trimmed, ""))
}

pub(crate) static UNESCAPING_AC: LazyLock<AhoCorasick> =
    LazyLock::new(|| AhoCorasick::new(["&quot;", "&#x22;", "&#x27;"]).unwrap());

/// Width of the smallest indentation among non-blank lines, in columns.
/// a space = a column, a tab = tab_width columns
pub(crate) fn detect_indent<'a>(lines: impl Iterator<Item = &'a str>, tab_width: usize) -> usize {
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.chars()
                .take_while(|c| matches!(c, ' ' | '\t'))
                .map(|c| if c == '\t' { tab_width } else { 1 })
                .sum()
        })
        .min()
        .unwrap_or_default()
}

/// Drop `indent` columns of indentation, re-emitting a tab straddling the cut as spaces.
pub(crate) fn strip_indent(line: &str, indent: usize, tab_width: usize) -> Cow<'_, str> {
    let mut width = 0;
    for (i, c) in line.char_indices() {
        if width == indent {
            return Cow::from(&line[i..]);
        }
        width += match c {
            ' ' => 1,
            '\t' => tab_width,
            _ => return Cow::from(&line[i..]),
        };
        if width > indent {
            return Cow::from(format!(
                "{}{}",
                " ".repeat(width - indent),
                &line[i + c.len_utf8()..]
            ));
        }
    }
    Cow::from("")
}

pub(crate) fn pascal2kebab(s: &'_ str) -> Cow<'_, str> {
    let uppers = s.chars().filter(char::is_ascii_uppercase).count();
    if uppers > 1
        || s.find(|c: char| c.is_ascii_uppercase())
            .is_some_and(|index| index > 0)
    {
        let mut result = String::with_capacity(s.len() + uppers);
        s.chars().fold('<', |prev, c| {
            if c.is_ascii_uppercase() && prev.is_ascii_alphanumeric() {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
            c
        });
        Cow::from(result)
    } else {
        Cow::from(s)
    }
}

pub(crate) fn kebab2pascal(s: &'_ str) -> Cow<'_, str> {
    if s.contains('-')
        || s.find(|c: char| c.is_ascii_uppercase())
            .is_some_and(|index| index > 0)
    {
        let mut result = String::with_capacity(s.len());
        s.chars().fold('<', |prev, c| {
            if c == '-' {
            } else if matches!(prev, '-' | '<') {
                result.push(c.to_ascii_uppercase());
            } else {
                result.push(c);
            }
            c
        });
        Cow::from(result)
    } else {
        Cow::from(s)
    }
}

pub(crate) fn has_template_interpolation(s: &str, language: Language) -> bool {
    match language {
        Language::Html | Language::Xml => false,
        Language::Svelte | Language::Astro => s.contains('{'),
        Language::Vue | Language::Angular => s.contains("{{"),
        Language::Jinja | Language::Django | Language::Vento | Language::Mustache => {
            s.contains("{{") || s.contains("{%")
        }
    }
}

static SPACE_SEPARATED_GLOBAL_ATTRIBUTES: [&str; 11] = [
    "class",
    "aria-labelledby",
    "aria-describedby",
    "aria-controls",
    "aria-owns",
    "aria-flowto",
    "accesskey",
    "itemtype",
    "itemprop",
    "itemref",
    "accesskey",
];
/// Checks if the given attribute name content should be space-separated.
///
/// These were found using the HTML attribute list from the spec, cross-referencing MDN:
/// - <https://html.spec.whatwg.org/multipage/indices.html#attributes-3>
/// - <https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes>
pub(crate) fn should_be_space_separated(attr_name: &str, tag_name: Option<&str>) -> bool {
    if SPACE_SEPARATED_GLOBAL_ATTRIBUTES
        .iter()
        .any(|tag| tag.eq_ignore_ascii_case(attr_name))
    {
        true
    } else if attr_name.eq_ignore_ascii_case("rel") {
        tag_name.is_some_and(|name| {
            ["form", "a", "area", "link"]
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(name))
        })
    } else if attr_name.eq_ignore_ascii_case("blocking") {
        tag_name.is_some_and(|name| {
            ["link", "script", "style"]
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(name))
        })
    } else if attr_name.eq_ignore_ascii_case("for") {
        tag_name.is_some_and(|name| name.eq_ignore_ascii_case("output"))
    } else if attr_name.eq_ignore_ascii_case("headers") {
        tag_name.is_some_and(|name| {
            ["td", "th"]
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(name))
        })
    } else if attr_name.eq_ignore_ascii_case("autocomplete") {
        tag_name.is_some_and(|name| {
            ["form", "input", "select", "textarea"]
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(name))
        })
    } else if attr_name.eq_ignore_ascii_case("sandbox") {
        tag_name.is_some_and(|name| name.eq_ignore_ascii_case("iframe"))
    } else if attr_name.eq_ignore_ascii_case("accept-charset") {
        tag_name.is_some_and(|name| name.eq_ignore_ascii_case("form"))
    } else if attr_name.eq_ignore_ascii_case("ping") {
        tag_name.is_some_and(|name| {
            ["a", "area"]
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(name))
        })
    } else {
        false
    }
}

pub(crate) fn pos_to_line_col(source: &str, pos: usize) -> (usize, usize) {
    let search = memchr::memchr_iter(b'\n', source.as_bytes()).try_fold(
        (1, 0),
        |(line, prev_offset), offset| match pos.cmp(&offset) {
            Ordering::Less => ControlFlow::Break((line, prev_offset)),
            Ordering::Equal => ControlFlow::Break((line, prev_offset)),
            Ordering::Greater => ControlFlow::Continue((line + 1, offset)),
        },
    );
    match search {
        ControlFlow::Break((line, offset)) => (line, pos - offset + 1),
        ControlFlow::Continue((line, _)) => (line, 0),
    }
}

/// Why a comment addressed to a directive namespace is no directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// The keyword after the namespace is not one the caller honors.
    UnknownKeyword,
    /// An empty `[]` list.
    MissingCodes,
    MissingBracket,
    MissingComma,
    /// A code must start with a letter.
    InvalidCode,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownKeyword => "unknown directive",
            Self::MissingCodes => "missing suppression codes like `[code, ...]`",
            Self::MissingBracket => "missing closing bracket",
            Self::MissingComma => "missing comma between codes",
            Self::InvalidCode => "invalid code",
        })
    }
}

impl std::error::Error for ParseErrorKind {}

/// A `namespace:keyword[code, ...]` directive read from a comment body.
#[derive(Debug, PartialEq, Eq)]
pub struct Directive<'s> {
    pub keyword: &'s str,
    /// The `[...]` codes; a bare directive lists none.
    pub codes: Vec<&'s str>,
}

/// Parse a comment body as a `namespace:keyword[code, ...]` directive honoring `keywords`.
/// `None` is a comment not addressed to `namespace` at all.
/// An empty namespace reads a bare `keyword[code, ...]`.
///
/// Whitespace is tolerated around the colon and before the list,
/// a bare keyword may be followed by free text, and so may the closing bracket.
/// Jinja's whitespace-control markers (`{#- ... -#}`) belong to the delimiter and are skipped.
pub fn parse_directive<'s>(
    comment: &'s str,
    namespace: &str,
    keywords: &[&str],
) -> Option<Result<Directive<'s>, ParseErrorKind>> {
    // This runs on every comment of every file, so a prose comment must fail on its first byte,
    // before anything reads the rest of the body.
    let mut rest = comment
        .trim_start()
        .trim_start_matches(['-', '+'])
        .trim_start();
    if !namespace.is_empty() {
        rest = rest
            .strip_prefix(namespace)?
            .trim_start()
            .strip_prefix(':')?;
    }
    let rest = rest.trim().trim_end_matches('-');
    let keyword_end = rest
        .find(|c: char| c == '[' || c.is_whitespace())
        .unwrap_or(rest.len());
    let (keyword, rest) = rest.split_at(keyword_end);
    if !keywords.contains(&keyword) {
        return Some(Err(ParseErrorKind::UnknownKeyword));
    }
    let codes = rest
        .trim_start()
        .strip_prefix('[')
        .map_or(Ok(Vec::new()), parse_codes);
    Some(codes.map(|codes| Directive { keyword, codes }))
}

/// The codes of a `[...]` list, `list` starting right after the bracket.
fn parse_codes(list: &str) -> Result<Vec<&str>, ParseErrorKind> {
    let mut codes = Vec::new();
    let mut rest = list.trim_start();
    loop {
        if rest.is_empty() {
            return Err(ParseErrorKind::MissingBracket);
        }
        if rest.starts_with(']') {
            break;
        }
        let (code, after) = rest.split_at(code_len(rest));
        if code.is_empty() {
            return Err(ParseErrorKind::InvalidCode);
        }
        codes.push(code);
        rest = after.trim_start();
        match rest.strip_prefix(',') {
            Some(after_comma) => rest = after_comma.trim_start(),
            None if rest.starts_with(']') => break,
            None if rest.is_empty() => return Err(ParseErrorKind::MissingBracket),
            None => return Err(ParseErrorKind::MissingComma),
        }
    }
    if codes.is_empty() {
        return Err(ParseErrorKind::MissingCodes);
    }
    Ok(codes)
}

/// How far a code runs from the start of `text`: a letter, then letters, digits, `_`, `-`,
/// or a `:` tolerated so `lint:code` still reads as one code.
fn code_len(text: &str) -> usize {
    let mut chars = text.char_indices();
    if !chars.next().is_some_and(|(_, c)| c.is_alphabetic()) {
        return 0;
    }
    chars
        .find(|(_, c)| !(c.is_alphanumeric() || matches!(c, '_' | '-' | ':')))
        .map_or(text.len(), |(index, _)| index)
}

/// Whether `comment` matches the configured `directive`: a bare entry such as
/// `markup-fmt:ignore` matches only a bare comment, while a `markup-fmt:ignore[format]`
/// entry matches when the comment's code list contains that code.
pub fn matches_directive(comment: &str, directive: &str) -> bool {
    let (name, required) = match directive.split_once('[') {
        Some((name, rest)) => match rest.strip_suffix(']') {
            Some(code) => (name, Some(code)),
            // A configured entry missing its `]` names no code, so it matches nothing.
            None => return false,
        },
        None => (directive, None),
    };
    // `markup-fmt:ignore` splits into a namespace and a keyword, `markup-fmt-ignore` is all keyword.
    let (namespace, keyword) = name.rsplit_once(':').unwrap_or(("", name));
    match (parse_directive(comment, namespace, &[keyword]), required) {
        (Some(Ok(parsed)), None) => parsed.codes.is_empty(),
        (Some(Ok(parsed)), Some(code)) => parsed.codes.contains(&code),
        _ => false,
    }
}

/// Joins `statics` with numbered placeholders (`<placeholder><slot>_`), so a formatter that
/// reorders or drops a declaration cannot shift the interpolations onto the wrong slots.
pub(crate) fn mask_interpolations(statics: &[&str], placeholder: &str) -> String {
    let Some((first, rest)) = statics.split_first() else {
        return String::new();
    };
    let mut masked = String::from(*first);
    for (slot, text) in rest.iter().enumerate() {
        masked.push_str(placeholder);
        masked.push_str(&slot.to_string());
        masked.push('_');
        masked.push_str(text);
    }
    masked
}

#[cfg(test)]
mod tests {
    use super::ParseErrorKind;
    use rstest::rstest;
    use std::iter;

    #[rstest]
    fn matching_directive(
        #[values(
            "markup-fmt:ignore",
            "markup-fmt: ignore",
            "markup-fmt :ignore",
            "markup-fmt \t:\t ignore",
            "  markup-fmt:ignore",
            "\n markup-fmt:ignore",
            "markup-fmt:ignore ",
            "markup-fmt:ignore\nmore text",
            "markup-fmt:ignore why not",
            // Jinja whitespace-control markers belong to the delimiter.
            "- markup-fmt:ignore -",
            "markup-fmt:ignore-",
            "+ markup-fmt:ignore"
        )]
        comment: &str,
    ) {
        assert!(super::matches_directive(comment, "markup-fmt:ignore"));
    }

    #[rstest]
    fn non_matching_directive(
        #[values(
            "",
            "markup-fmt",
            "markup-fmt:",
            "markup-fmt:ignor",
            "markup-fmt:ignore-file",
            "markup-fmt::ignore",
            "markup-fmt ignore",
            "MARKUP-FMT:IGNORE",
            ":ignore",
            "prefix markup-fmt:ignore",
            "markup-fmt:i gnore",
            // A code list is a directive, but not a bare one.
            "markup-fmt:ignore[a]"
        )]
        comment: &str,
    ) {
        assert!(!super::matches_directive(comment, "markup-fmt:ignore"));
    }

    #[rstest]
    #[case::single("markup-fmt:ignore[a]", vec!["a"])]
    #[case::several("markup-fmt : ignore[a, b ,c]", vec!["a", "b", "c"])]
    #[case::spaced_list("markup-fmt:ignore [a]", vec!["a"])]
    #[case::trailing_comma("markup-fmt:ignore[a,]", vec!["a"])]
    #[case::reason("markup-fmt:ignore[a]: why not", vec!["a"])]
    #[case::markers("- markup-fmt:ignore[a] -", vec!["a"])]
    #[case::bare_with_reason("markup-fmt:ignore why [not]", vec![])]
    fn directive_code_lists(#[case] comment: &str, #[case] expected: Vec<&str>) {
        let parsed = super::parse_directive(comment, "markup-fmt", &["ignore"]);
        assert_eq!(
            parsed.expect("a directive").expect("well formed").codes,
            expected
        );
    }

    #[rstest]
    #[case::unknown_keyword("markup-fmt:ignor[a]", ParseErrorKind::UnknownKeyword)]
    #[case::empty_list("markup-fmt:ignore[]", ParseErrorKind::MissingCodes)]
    #[case::unclosed_list("markup-fmt:ignore[a", ParseErrorKind::MissingBracket)]
    #[case::missing_comma("markup-fmt:ignore[a b]", ParseErrorKind::MissingComma)]
    #[case::separators_only("markup-fmt:ignore[ , ]", ParseErrorKind::InvalidCode)]
    #[case::numeric_code("markup-fmt:ignore[1x]", ParseErrorKind::InvalidCode)]
    fn malformed_directives(#[case] comment: &str, #[case] expected: ParseErrorKind) {
        assert_eq!(
            super::parse_directive(comment, "markup-fmt", &["ignore"]),
            Some(Err(expected))
        );
    }

    #[rstest]
    // A bare configured entry ignores comments carrying a code list, and vice versa.
    #[case("markup-fmt:ignore", "markup-fmt:ignore", true)]
    #[case("markup-fmt:ignore[format]", "markup-fmt:ignore", false)]
    #[case("markup-fmt:ignore", "markup-fmt:ignore[format]", false)]
    // A code entry matches on membership, whatever the order or spacing.
    #[case("markup-fmt:ignore[format]", "markup-fmt:ignore[format]", true)]
    #[case("markup-fmt:ignore [format]", "markup-fmt:ignore[format]", true)]
    #[case("markup-fmt:ignore[ format ]", "markup-fmt:ignore[format]", true)]
    #[case("markup-fmt:ignore[a, format]", "markup-fmt:ignore[format]", true)]
    #[case("markup-fmt:ignore[format, a]", "markup-fmt:ignore[format]", true)]
    #[case("markup-fmt:ignore[a]", "markup-fmt:ignore[format]", false)]
    // A malformed list is no directive at all, so it cannot opt a node out.
    #[case("markup-fmt:ignore[format", "markup-fmt:ignore[format]", false)]
    #[case("markup-fmt:ignore[format", "markup-fmt:ignore", false)]
    // A colon-less entry is a keyword on its own.
    #[case("markup-fmt-ignore", "markup-fmt-ignore", true)]
    #[case("markup-fmt-ignore-file", "markup-fmt-ignore", false)]
    #[case("markup-fmt-ignore[format]", "markup-fmt-ignore[format]", true)]
    fn configured_directive_entries(
        #[case] comment: &str,
        #[case] configured: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(super::matches_directive(comment, configured), expected);
    }

    #[test]
    fn pos_to_line_col() {
        let source = "abc\ndef\nghi";
        // Positions whose line is followed by a later newline go through the
        // `Break` arm, which must report a non-zero column.
        assert_eq!(super::pos_to_line_col(source, 0), (1, 1));
        assert_eq!(super::pos_to_line_col(source, 2), (1, 3));
        assert_eq!(super::pos_to_line_col(source, 4), (2, 2));
        assert_eq!(super::pos_to_line_col(source, 6), (2, 4));
    }

    #[test]
    fn detect_indent() {
        assert_eq!(super::detect_indent(["\tb", "  c"].into_iter(), 4), 2);
        assert_eq!(super::detect_indent(["  a", "", " b"].into_iter(), 4), 1);
        // every line filtered out by the caller, e.g. an all-continuation script
        assert_eq!(super::detect_indent(iter::empty::<&str>(), 4), 0);
    }

    #[test]
    fn strip_indent() {
        assert_eq!(super::strip_indent("      a", 4, 4), "  a");
        assert_eq!(super::strip_indent("\t\ta", 4, 4), "\ta");
        assert_eq!(super::strip_indent("a", 4, 4), "a");
        assert_eq!(super::strip_indent("   ", 4, 4), "");
        assert_eq!(super::strip_indent("\ta", 2, 4), "  a");
    }

    #[test]
    fn each_interpolation_gets_a_numbered_slot() {
        let masked = super::mask_interpolations(&["a: url(", "); b: ", "px"], "_ph_");
        assert_eq!(masked, "a: url(_ph_0_); b: _ph_1_px");
    }
}

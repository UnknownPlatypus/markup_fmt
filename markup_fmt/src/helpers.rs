use crate::state::State;
use crate::Language;
use aho_corasick::AhoCorasick;
use std::sync::LazyLock;

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
                    && !css_dataset::tags::SVG_TAGS
                        .iter()
                        .any(|tag| tag.eq_ignore_ascii_case(name))
        }
        Language::Xml => true,
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

pub(crate) fn detect_indent(s: &str) -> usize {
    s.lines()
        .skip(if s.starts_with([' ', '\t']) { 0 } else { 1 })
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.as_bytes()
                .iter()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count()
        })
        .min()
        .unwrap_or_default()
}

/// Checks if the given attribute name content should be space-separated.
///
/// These were found using the HTML attribute list, cross-referencing the HTML spec:
/// - <https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes>
/// - <https://html.spec.whatwg.org/multipage/>
pub(crate) fn should_be_space_separated(name: &str, state: &State) -> bool {
    name.eq_ignore_ascii_case("class")
        || name.eq_ignore_ascii_case("aria-labelledby")
        || name.eq_ignore_ascii_case("aria-describedby")
        || name.eq_ignore_ascii_case("aria-controls")
        || name.eq_ignore_ascii_case("aria-owns")
        || name.eq_ignore_ascii_case("rel")
            && state
                .current_tag_name
                .map(|name| {
                    ["form", "a", "area", "link"]
                        .iter()
                        .any(|tag| tag.eq_ignore_ascii_case(name))
                })
                .unwrap_or_default()
        || name.eq_ignore_ascii_case("autocomplete")
            && state
                .current_tag_name
                .map(|name| {
                    ["form", "input", "select", "textarea"]
                        .iter()
                        .any(|tag| tag.eq_ignore_ascii_case(name))
                })
                .unwrap_or_default()
        || name.eq_ignore_ascii_case("sandbox")
            && state
                .current_tag_name
                .map(|name| name.eq_ignore_ascii_case("iframe"))
                .unwrap_or_default()
        || name.eq_ignore_ascii_case("accept-charset")
            && state
                .current_tag_name
                .map(|name| name.eq_ignore_ascii_case("form"))
                .unwrap_or_default()
}

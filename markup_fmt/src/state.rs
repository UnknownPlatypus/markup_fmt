#[derive(Clone)]
pub(crate) struct State<'s> {
    pub(crate) current_tag_name: Option<&'s str>,
    pub(crate) is_root: bool,
    pub(crate) in_svg: bool,
    pub(crate) indent_level: u16,
    /// Inside a `{% for %}` block used as an attribute, where body-edge whitespace
    /// separates loop iterations.
    pub(crate) in_attr_loop: bool,
    /// Inside a trimmed translation block, which turns every line break into a space,
    /// so markup there can only break where the source already has whitespace.
    pub(crate) in_trimmed_translation: bool,
}

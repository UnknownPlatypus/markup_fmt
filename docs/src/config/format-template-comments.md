# `formatTemplateComments`

Control whether template comments (`{# ... #}`) are formatted
even when [`formatComments`](./format-comments.md) is `false`. HTML comments are left untouched.

In Django templates, a comment gets exactly one space at its beginning and end and its content is kept as-is.
Only single-line comments are affected: Django renders a `{# ... #}` spanning several lines as text.
A comment whose text starts or ends with `-` is left as written, since Django has no whitespace control
and the dashes are part of the comment.

In Jinja templates, comments are formatted as [`formatComments`](./format-comments.md) does.

Default option is `false`.

## Example for `false`

```html
{#comments#}
{#   comments   #}
```

will be formatted as its original input.

## Example for `true`

```html
{# comments #}
{# comments #}
```

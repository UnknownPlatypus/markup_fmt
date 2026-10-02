# `formatTemplateComments`

Control whether Django template comments (`{# ... #}`) get exactly one space at their beginning and end,
even when [`formatComments`](./format-comments.md) is `false`.
Their content is kept as-is, and HTML comments are left untouched.

Only single-line comments are affected: Django renders a `{# ... #}` spanning several lines as text.

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

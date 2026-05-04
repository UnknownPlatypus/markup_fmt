# preserveUnquotedAttrs

Control whether unquoted attribute values are left unquoted instead of being wrapped in quotes.

- Type: `boolean`
- Default: `false`

When set to `true`, attribute values that were originally unquoted in the source will remain unquoted in the formatted output. When `false` (default), all attribute values will be wrapped in quotes according to the `quotes` option.

This is useful for template languages like Jinja/Django where attribute values may contain template expressions that should not be quoted.

## Example

Input:

```html
<a href={{ url }}>link</a>
<a href="quoted">link</a>
```

With `preserveUnquotedAttrs: false` (default):

```html
<a href="{{ url }}">link</a>
<a href="quoted">link</a>
```

With `preserveUnquotedAttrs: true`:

```html
<a href={{ url }}>link</a>
<a href="quoted">link</a>
```

# `formatComments`

Control whether whitespace should be inserted at the beginning and end of comments and
comments should be indented properly or not.

When this option is set to `false`, comments contain leading or trailing whitespace will still be kept as-is.

A Django `{# ... #}` spanning several lines is text to Django, so it is kept as-is either way.

Default option is `false`.

## Example for `false`

```html
<!--comments-->
<!-- comments -->
<!--very very very very very very very very very very very very very very very very long-->
```

will be formatted as its original input.

## Example for `true`

```html
<!-- comments -->
<!-- comments -->
<!--
  very very very very very very very very very very very very very very very very long
-->
```

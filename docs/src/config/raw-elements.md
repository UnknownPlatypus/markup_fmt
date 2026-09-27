# rawElements

Elements whose content is kept byte for byte, like `<pre>`.

- Type: `string[]`
- Default: `[]`

The content is parsed as raw text, so unbalanced HTML inside is fine.
The opening tag's attributes are still formatted.
The element closes at the first `</name>`, so nesting the same element isn't supported.
Names match case-insensitively.

## Example

With `rawElements: ["c-markdown"]`, this input is left untouched:

```html
<c-markdown>
# Title

- item   one
- item two
</c-markdown>
```

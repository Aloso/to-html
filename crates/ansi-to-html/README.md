# ansi-to-html

[Documentation](https://docs.rs/crate/ansi-to-html)

Rust library to convert a string that can contain [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code) to HTML.

## ANSI support

This crate currently supports SGR parameters (text style and colors).
The supported styles are:

- bold
- italic
- underlined
- doubly underlined
- overlined
- crossed out
- faint
- reverse video
- foreground and background colors: 3-bit, 4-bit, 8-bit, truecolor (24-bit)

**Not** supported SGR parameters (note that most of these are niche features
and rarely supported by terminals):

- slow/rapid blink
- conceal
- alternative fonts
- fraktur
- proportional spacing
- framed
- encircled
- ideogram attributes
- non-standard extensions
  - underline color
  - superscript, subscript
  - bright foreground/background color

All unsupported ANSI escape codes are stripped from the output.

It should be easy to add support for more styles, if there's a straightforward HTML
representation. If you need a different style (e.g. doubly underlined), file an issue.

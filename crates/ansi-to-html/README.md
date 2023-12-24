# ansi-to-html

[Documentation](https://docs.rs/crate/ansi-to-html)

Rust library to convert a string that can contain [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code) to HTML.

## ANSI support

This crate currently supports SGR parameters (text style and colors).
The supported styles are:

- bold
- italic
- underlined
- reverse video
- crossed out
- faint
- foreground and background colors: 3-bit, 4-bit, 8-bit, truecolor (24-bit)

**Not** supported SGR parameters (note that most of these are niche features
and rarely supported by terminals):

- slow/rapid blink
- conceal
- alternative fonts
- fraktur
- doubly underlined
- proportional spacing
- framed
- encircled
- overlined
- underline color (not in standard)
- ideogram attributes
- superscript, subscript (not in standard)
- bright foreground/background color (not in standard)

All unsupported ANSI escape codes are stripped from the output.

It should be easy to add support for more styles, if there's a straightforward HTML
representation. If you need a different style (e.g. doubly underlined), file an issue.

## Features

When the  `lazy-init` feature is enabled, regexes are lazily initialized, which is more efficient if you want to convert A LOT of strings. The performance difference has not been tested.

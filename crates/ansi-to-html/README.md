# ansi-to-html

[Documentation](https://docs.rs/crate/ansi-to-html)

Rust library to convert a string that can contain [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code) to HTML.

## ANSI support

This crate currently supports SGR parameters (text style and colors).
The supported styles are:

| style | `\e[{CODE}m` | sample | `convert(sample)` |
| :---: | :---: | :---: | :--- |
| Bold | 1 | `\e[1mBold` | `<b>Bold</b>` |
| Faint | 2 | `\e[2mFaint` | `<span style='opacity:0.67'>Faint</span>` |
| Italic | 3 | `\e[3mItalic` | `<i>Italic</i>` |
| Underlined | 4 | `\e[4mUnderlined` | `<u>Underlined</u>` |
| Doubly Underlined | 21 | `\e[21mDouble` | `<u style='text-decoration-style:double'>Double</u>` |
| Overlined | 53 | `\e[53mOverlined` | `<u style='text-decoration:overline'>Overlined</u>` |
| Crossed Out | 9 | `\e[9mStriked` | `<s>Striked</s>` |
| Reverse Video | 7 | `\e[7mReverse` | `<span style='color:var(--black,#000);background:var(--bright-white,#fff)'>Reverse</span>` |
| 3/4-bit fg/bg color | 30-37, 40-47, 90-97, 100-107 | `\e[31mRed` | `<span style='color:var(--red,#a00)'>Red</span>` |
| 8-bit fg/bg color | `38;5;{NUM}`, `48;5;{NUM}` | `\e[38;5;211m#211` | `<span style='color:#ff87af'>#211</span>` |
| fg/bg truecolor (24-bit) | `38;2;{R};{G};{B}`, `48;2;{R};{G};{B}` | `\e[38;2;224;176;255mMauve` | `<span style='color:#e0b0ff'>Mauve</span>` |

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

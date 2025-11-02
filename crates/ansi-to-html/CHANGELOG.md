# Changelog

## [0.2.3] - Unreleased

There's quite a lot packed into this release. The headliners are:

- Support for more ANSI codes (double underline, overline, and reverse video)
  - and support for reverse video involved adding a new `Theme` and
    `Converter::theme` API
- The regex-based ANSI parser was replaced with an optimized handwritten version
- Escaping text was sped up significantly
- There's a fuzz test now (and it already found a couple of bugs)
- There are benchmarks now...

The benchmark measures the throughput of converting either plain text or ANSI
text consisting primarily of a very warning heavy `cargo clippy` run with a
variety of different options set. Lets see how we stack up compared to the
`0.2.2` release (measured with `$ DIVAN_MIN_TIME=5 cargo bench` and manually
reformatted)

_Note: the default options for `convert()` are to both escape and optimize_

**ANSI heavy input**

| Esc | Opt | 0.2.2 | 0.2.3 |
| :--: | :--: | --: | --: |
| :heavy_check_mark: | :heavy_check_mark: | 69.0 MB/s | 112.9 MB/s |
| :heavy_check_mark: | :x: | 81.5 MB/s | 168.3 MB/s |
| :x: | :heavy_check_mark: | 90.5 MB/s | 140.4 MB/s |
| :x: | :x: | 114.3 MB/s | 205.8 MB/s |

**Plain text input**

| Esc | Opt | 0.2.2 | 0.2.3 |
| :--: | :--: | --: | --: |
| :heavy_check_mark: | :heavy_check_mark: | 249.8 MB/s | 971.8 MB/s |
| :heavy_check_mark: | :x: | 233.4 MB/s | 990.9 MB/s |
| :x: | :heavy_check_mark: | 9.2 GB/s | 9.6 GB/s |
| :x: | :x: | 31.3 GB/s | 32.6 GB/s |

Significant improvements across the board :tada:

### Features

- Change ANSI code 21 from _bold off_ to _double underline_ (thereby adding
  support for _double underline_) [`#44`]
- Add support for the _reverse video_ ANSI code [`#46`]
- Mark several `Converter` methods as `#[must_use]` [`#115`]
- Add support for the _overline_ ANSI code [`#123`]
- Parse and ignore Operating System Command (OSC) sequences [`#91`]

### Fixes

- Allow applying already applied styles [`bd6f7fa`]
  - Previously a sequence like _red_ -> _blue_ -> _red_ would ignore the second
    _red_
- Maintain the same order when removing styles from the stack [`2cfa051`]
  - Previously styles in the stack applied after the style to be removed would
    be pushed back on in reverse. This means a sequence like _underline_ ->
    _red_ -> _blue_ -> _underline off_ would become _blue_ -> _red_ instead of
    _red_ -> _blue_
- Interpret overlapping colors in 256- and 16- color palettes as 16-color
  [`#62`]

### Deps

- Drop `thiserror` in favor of a manual implementation [`#40`]

### Docs

- Add a set of benchmarks [`#39`] and expand on them [`#49`]
- Add a simple `ansi2html` pipe example [`#114`]

### Internal

- Add a fuzz test to ensure that optimized output is semantically equivalent
  to the unoptimized output [`#41`]
- Replace the regex based parser for a handwritten one [`#45`] and improved its
  performance _considerably_ [`#82`]
- Speed up escaping by working on byte chunks instead of iterating over each
  char [`#63`]
- ... and many many more changes that I'm too lazy to list here :)

[`#39`]: https://github.com/Aloso/to-html/pull/39
[`#40`]: https://github.com/Aloso/to-html/pull/40
[`#41`]: https://github.com/Aloso/to-html/pull/41
[`#44`]: https://github.com/Aloso/to-html/pull/44
[`#45`]: https://github.com/Aloso/to-html/pull/45
[`#46`]: https://github.com/Aloso/to-html/pull/46
[`#49`]: https://github.com/Aloso/to-html/pull/49
[`#62`]: https://github.com/Aloso/to-html/pull/62
[`#63`]: https://github.com/Aloso/to-html/pull/63
[`#82`]: https://github.com/Aloso/to-html/pull/82
[`#91`]: https://github.com/Aloso/to-html/pull/91
[`#114`]: https://github.com/Aloso/to-html/pull/114
[`#115`]: https://github.com/Aloso/to-html/pull/115
[`#123`]: https://github.com/Aloso/to-html/pull/123
[`bd6f7fa`]: https://github.com/Aloso/to-html/commit/bd6f7fa1340d1e9c988ac446ad4e8d284219dd5f
[`2cfa051`]: https://github.com/Aloso/to-html/commit/2cfa0513000cc31914c8bf001b4351b3d9e57640

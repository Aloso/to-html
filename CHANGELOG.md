# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.6] - 2024-11-29

- [`#34`](https://github.com/Aloso/to-html/pull/34): Allows for a single optional semicolon in ansi codes before the terminating `m` (@CosmicHorrorDev)

- [`df3d898`](https://github.com/Aloso/to-html/commit/df3d898a383d3f05a8ff9d769052fb6d3b0370ab):
  Auto-detect shell from parent process name (@Aloso)

- [`b1e4774`](https://github.com/Aloso/to-html/commit/b1e4774200ee557be292531c5af30e88a3f875d5):
  Make all values in config file optional (@Aloso)

### Internal improvements

- [`#33`](https://github.com/Aloso/to-html/pull/33): Replace `once_cell` usage with `std::sync::OnceLock` (@CosmicHorrorDev)

- [`#35`](https://github.com/Aloso/to-html/pull/35): CI tweaks (@CosmicHorrorDev)

- [`#36`](https://github.com/Aloso/to-html/pull/36): Add builder API to `ansi-to-html` crate (@Aloso)

## [0.1.5] - 2023-11-30

- [`#19`](https://github.com/Aloso/to-html/pull/19): Make `to-html` configurable with a config file located in
  the proper location depending on the OS (@CosmicHorrorDev)

  - Linux: `$XDG_CONFIG_HOME/to-html/config.toml` or `~/.config/to-html/config.toml`
  - macOS: `$HOME/Library/Application Support/to-html/config.toml`

  An example configuration file is [here](/config.toml.sample).

- [`#22`](https://github.com/Aloso/to-html/pull/22): Add shell completions to release builds (@CosmicHorrorDev)

- [`#25`](https://github.com/Aloso/to-html/pull/25): Make terminal colors configurable via CSS classes
  (e.g. `--red`, `--bright-green`) (@CosmicHorrorDev)

- [`#28`](https://github.com/Aloso/to-html/pull/28): Add `--hide-prompt` flag to print the command output
  without the prompt (@Julian-Alberts)

- fix ([`0a5fcbb`](https://github.com/Aloso/to-html/commit/0a5fcbbfae27d13d51ebeca3c14915656bdf73c1)):
  Correctly parse backslash escapes like `\n` (@Aloso)

## [0.1.4] - 2023-03-29

- [`#18`](https://github.com/Aloso/to-html/pull/18): Minify output with redundant ANSI escape better (@CosmicHorrorDev)

- [`#15`](https://github.com/Aloso/to-html/pull/15): Internal: Update command-line argument parser (@CosmicHorrorDev)

- [`#12`](https://github.com/Aloso/to-html/pull/12): Internal: Improve continuous integration (@CosmicHorrorDev)

## [0.1.3] - 2022-09-08

- [`#6`](https://github.com/Aloso/to-html/pull/6): Support different shells.

  All shells can be used that have a `-c <command>` argument to execute a command, and support multiple commands separated with `;`. This includes `bash`, `fish`, `ksh` and `zsh`.

- [`#5`](https://github.com/Aloso/to-html/pull/5): Add `--doc`/`-d` flag to emit a full HTML document. It can then be redirected to a file like this:

  ```shell
  $ to-html 'cargo test --workspace' -d > output.html
  $ firefox output.html
  ```

## [0.1.1] - 2020-11-18

- [`#4`](https://github.com/Aloso/to-html/pull/4): FreeBSD support added (this also fixed a few bugs on macOS)

## [0.1.0] - 2020-11-15

Initial release
